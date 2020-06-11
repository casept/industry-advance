use super::background::LargeBackground;

use crate::shared_constants::SCREENBLOCK_SIZE_BYTES;
use crate::FS;

use core::str;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use gbfs_rs::Filename;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Map {
    bg: LargeBackground,
}

impl Map {
    /// Create a new map.
    /// `x` and `y` are the size of the entire map, given in number of horizontal and vertical 32x32 sub-tilemaps, respectively.
    /// . Their number must match x*y and they must be in the vector in a left-to-right, top-to-bottom order.
    /// Each tilemap must be SCREENBLOCK_SIZE_IN_U8 large.
    /// If it isn't, this function will panic.
    pub fn new_map(
        palette: &'static [u16],
        x_size_in_tilemaps: usize,
        y_size_in_tilemaps: usize,
        tiles: &'static [u32],
        tilemaps: Vec<&'static [u8]>,
    ) -> Map {
        for tilemap in tilemaps.clone() {
            assert_eq!(tilemap.len(), SCREENBLOCK_SIZE_BYTES);
        }
        let mut two_d_indexed_tilemaps: Vec<Vec<&'static [u8]>> =
            Vec::with_capacity(x_size_in_tilemaps);
        for i in 0..x_size_in_tilemaps {
            two_d_indexed_tilemaps.push(Vec::with_capacity(y_size_in_tilemaps));
            for j in 0..y_size_in_tilemaps {
                two_d_indexed_tilemaps[i].push(tilemaps[i * x_size_in_tilemaps + j]);
            }
        }
        let bg = LargeBackground::init(tiles, two_d_indexed_tilemaps, palette);
        return Map { bg };
    }

    /// Returns whether the given area (in pixels) is visible on screen right now.
    pub fn is_area_visible(
        &self,
        top_left_x: u32,
        top_left_y: u32,
        bottom_right_x: u32,
        bottom_right_y: u32,
    ) -> bool {
        return self
            .bg
            .is_area_visible(top_left_x, top_left_y, bottom_right_x, bottom_right_y);
    }

    // Returns the top-left corner (x, y) coordinates of the currently visible map area.
    pub fn get_top_left_corner_coords(&self) -> (u32, u32) {
        return self.bg.get_top_left_corner_coords();
    }

    /// Scroll the map by xy pixels.
    pub fn scroll(&mut self, x: i32, y: i32) {
        if x != 0 || y != 0 {
            self.bg.scroll(x, y);
        }
    }
}

/// Top-level struct describing all available maps.
#[derive(Deserialize)]
pub struct Maps {
    pub maps: Vec<MapEntry>,
}

/// Describes single map.
#[derive(Deserialize, Clone)]
pub struct MapEntry {
    name: String,
    height: usize,
    width: usize,
    chunks: Vec<MapChunk>,
}

/// Describes a 32x32 chunk.
#[derive(Deserialize, Clone)]
pub struct MapChunk {
    filename: String,
}

impl Maps {
    const MAPS_PATH: &'static str = "maps.json";
    const MAPS_SHARED_PAL: &'static str = "map_sharedPal";
    const MAPS_SHARED_TILES: &'static str = "map_sharedTiles";

    /// Reads from the default map description file.
    pub fn read_map_data() -> Maps {
        let map = FS
            .get_file_data_by_name(Filename::try_from_str(Maps::MAPS_PATH).unwrap())
            .unwrap();
        let map_data: Maps = serde_json::from_str(str::from_utf8(map).unwrap()).unwrap();
        map_data
    }
}

impl MapEntry {
    pub fn get_map(&self) -> Box<Map> {
        let mut tilemaps: Vec<&'static [u8]> = Vec::new();

        for chunk in &self.chunks {
            tilemaps.push(
                FS.get_file_data_by_name(Filename::try_from_str(&chunk.filename).unwrap())
                    .unwrap(),
            )
        }

        let pal: &'static [u16] = FS
            .get_file_data_by_name_as_u16_slice(
                Filename::try_from_str(Maps::MAPS_SHARED_PAL).unwrap(),
            )
            .unwrap();

        let tiles: &'static [u32] = FS
            .get_file_data_by_name_as_u32_slice(
                Filename::try_from_str(Maps::MAPS_SHARED_TILES).unwrap(),
            )
            .unwrap();

        let height = self.height / 32;
        let width = self.width / 32;
        return Box::new(Map::new_map(pal, width, height, tiles, tilemaps));
    }
}
