use std::{
    collections::{btree_map::Entry, BTreeMap},
    fs::{self, File},
    io::{self, ErrorKind, Read, Seek, SeekFrom},
    path::PathBuf,
};

use byteorder::{BigEndian, ReadBytesExt};
use flate2::bufread::{GzDecoder, ZlibDecoder};
use thiserror::Error;
use valence_nbt::Compound;
use vek::Vec3;

use crate::Seed;

const SECTOR_SIZE: usize = 4096;

fn get_region_coords_from_file_name(input: &str) -> (i32, i32) {
    let mut split = input.split('.');
    split.next(); // skip r.
    let x = split.next().unwrap().parse::<i32>().unwrap();
    let z = split.next().unwrap().parse::<i32>().unwrap();
    (x, z)
}

pub trait WorldState {
    fn new(world_root: impl Into<PathBuf>) -> Self;
    fn read_spawnpoint(&mut self) -> Result<(), ReadChunkError>;
    fn read_all(&mut self) -> Result<(), ReadChunkError>;
    fn read_region(&mut self, region_x: i32, region_z: i32) -> Result<(), ReadChunkError>;
    fn read_chunk(&self, chunk_x: i32, chunk_z: i32) -> Result<Option<AnvilChunk>, ReadChunkError>;
}

pub struct PiquantWorld {
    /// Path to the "region" subdirectory in the world root.
    world_root: PathBuf,
    region_root: PathBuf,
    /// Maps region (x, z) positions to region files.
    regions: BTreeMap<(i32, i32), Region>,

    pub spawn: Option<Vec3<f64>>,
    pub seed: Option<Seed>,
}

impl WorldState for PiquantWorld {
    fn new(world_root: impl Into<PathBuf>) -> Self {
        let world_root = world_root.into();
        let mut region_root = world_root.clone();
        region_root.push("region");

        Self {
            world_root,
            region_root,
            regions: BTreeMap::new(),

            spawn: None,
            seed: None,
        }
    }

    fn read_spawnpoint(&mut self) -> Result<(), ReadChunkError> {
        let mut level_dat = self.world_root.clone();
        level_dat.push("level.dat");

        let mut file = File::open(level_dat)?;
        let mut data_buf: Vec<u8> = Vec::new();
        file.read_to_end(&mut data_buf)?;

        let r: &[u8] = data_buf.as_ref();

        let mut decompress_buf = vec![];

        // What compression does the chunk use?

        let mut z = GzDecoder::new(r);
        z.read_to_end(&mut decompress_buf)?;
        let mut nbt_slice = decompress_buf.as_slice();

        let (nbt, _) = valence_nbt::from_binary_slice(&mut nbt_slice)?;

        let level = nbt.get("Data");
        let level = level.unwrap().as_compound().unwrap();
        let spawn_x = level.get("SpawnX").unwrap().as_int();
        let spawn_y = level.get("SpawnY").unwrap().as_int();
        let spawn_z = level.get("SpawnZ").unwrap().as_int();

        self.spawn = Some(Vec3::new(
            *spawn_x.unwrap() as f64,
            *spawn_y.unwrap() as f64,
            *spawn_z.unwrap() as f64,
        ));

        Ok(())
    }

    fn read_all(&mut self) -> Result<(), ReadChunkError> {
        // get all files from region folder
        // for each file
        // run read_region

        let paths = fs::read_dir(self.region_root.clone())?;
        for path in paths {
            // get the filename
            // parse the filename (r.0.0.mca), extract the 0, 0

            let path = path?;
            let path = path.path();
            // as str
            let path = path.to_str().unwrap();

            let coords = get_region_coords_from_file_name(path);

            self.read_region(coords.0, coords.1)?;
        }

        Ok(())
    }

    /// Reads a chunk from the file system with the given chunk coordinates. If
    /// no chunk exists at the position, then `None` is returned.
    fn read_region(&mut self, region_x: i32, region_z: i32) -> Result<(), ReadChunkError> {
        match self.regions.entry((region_x, region_z)) {
            Entry::Vacant(ve) => {
                // Load the region file if it exists. Otherwise, the chunk is considered absent.

                let path = self
                    .region_root
                    .join(format!("r.{region_x}.{region_z}.mca"));

                let mut file = match File::options().read(true).write(true).open(path) {
                    Ok(file) => file,
                    Err(e) if e.kind() == ErrorKind::NotFound => return Ok(()),
                    Err(e) => return Err(e.into()),
                };

                let mut header = [0; SECTOR_SIZE * 2];

                file.read_exact(&mut header)?;

                ve.insert(Region { file, header })
            }
            Entry::Occupied(oe) => oe.into_mut(),
        };

        Ok(())
    }

    fn read_chunk(&self, chunk_x: i32, chunk_z: i32) -> Result<Option<AnvilChunk>, ReadChunkError> {
        let region = match self
            .regions
            .get(&(chunk_x.div_euclid(32), chunk_z.div_euclid(32)))
        {
            Some(region) => region,
            None => return Ok(None),
        };

        let chunk_idx = (chunk_x.rem_euclid(32) + chunk_z.rem_euclid(32) * 32) as usize;

        let location_bytes = (&region.header[chunk_idx * 4..]).read_u32::<BigEndian>()?;
        let timestamp = (&region.header[chunk_idx * 4 + SECTOR_SIZE..]).read_u32::<BigEndian>()?;

        if location_bytes == 0 {
            // No chunk exists at this position.
            return Ok(None);
        }

        let sector_offset = (location_bytes >> 8) as u64;
        let sector_count = (location_bytes & 0xff) as usize;

        if sector_offset < 2 {
            // If the sector offset was <2, then the chunk data would be inside the region
            // header. That doesn't make any sense.
            return Err(ReadChunkError::BadSectorOffset);
        }

        // Seek to the beginning of the chunk's data.

        let mut file = region.file.try_clone()?;

        file.seek(SeekFrom::Start(sector_offset * SECTOR_SIZE as u64))?;

        let exact_chunk_size = file.read_u32::<BigEndian>()? as usize;

        if exact_chunk_size > sector_count * SECTOR_SIZE {
            // Sector size of this chunk must always be >= the exact size.
            return Err(ReadChunkError::BadChunkSize);
        }

        let mut data_buf = vec![0; exact_chunk_size].into_boxed_slice();
        file.read_exact(&mut data_buf)?;

        let mut r = data_buf.as_ref();

        let mut decompress_buf = vec![];

        // What compression does the chunk use?
        let mut nbt_slice = match r.read_u8()? {
            // GZip
            1 => {
                let mut z = GzDecoder::new(r);
                z.read_to_end(&mut decompress_buf)?;
                decompress_buf.as_slice()
            }
            // Zlib
            2 => {
                let mut z = ZlibDecoder::new(r);
                z.read_to_end(&mut decompress_buf)?;
                decompress_buf.as_slice()
            }
            // Uncompressed
            3 => r,
            // Unknown
            b => return Err(ReadChunkError::UnknownCompressionScheme(b)),
        };

        let (data, _) = valence_nbt::from_binary_slice(&mut nbt_slice)?;

        if !nbt_slice.is_empty() {
            return Err(ReadChunkError::IncompleteNbtRead);
        }

        Ok(Some(AnvilChunk { data, timestamp }))
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct AnvilChunk {
    /// This chunk's NBT data.
    pub data: Compound,
    /// The time this chunk was last modified measured in seconds since the
    /// epoch.
    pub timestamp: u32,
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ReadChunkError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Nbt(#[from] valence_nbt::Error),
    #[error("invalid chunk sector offset")]
    BadSectorOffset,
    #[error("invalid chunk size")]
    BadChunkSize,
    #[error("unknown compression scheme number of {0}")]
    UnknownCompressionScheme(u8),
    #[error("not all chunk NBT data was read")]
    IncompleteNbtRead,
}

#[derive(Debug)]
struct Region {
    file: File,
    /// The first 8 KiB in the file.
    header: [u8; SECTOR_SIZE * 2],
}
