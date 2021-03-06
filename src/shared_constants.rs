//! Constants used in multiple modules.

/// Screen height in pixels
pub const SCREEN_HEIGHT: usize = 160;
/// Screen width in pixels
pub const SCREEN_WIDTH: usize = 240;
/// Screen height in tiles
pub const SCREEN_HEIGHT_TILES: usize = SCREEN_HEIGHT / 8;
/// Screen width in tiles
pub const SCREEN_WIDTH_TILES: usize = SCREEN_WIDTH / 8;
/// Size of a screenblock in bytes
pub const SCREENBLOCK_SIZE_BYTES: usize = 32 * 32 * 2;
// Size of a single charblock in bytes
pub const CHARBLOCK_SIZE_BYTES: usize = 16 * 1024;
/// Size of a single tile edge in pixels
pub const TILE_SIZE_IN_PX: usize = 8;
/// Length of the edge of a single backing tilemap part in tiles
pub const BACKING_MAP_LENGTH_IN_TILES: usize = 32;
/// Charblock to use for map tiles
/// NOTE: Do not use for anything else!
pub const BACKGROUND_CHARBLOCK: usize = 0;
/// Screenblock to start at for loading backing tilemaps of map background
/// NOTE: Do not use for anything else!
pub const BACKGROUND_SCREEN_BASE_BLOCK: usize = 8;
/// Charblock used for font data
/// NOTE: Do not use for anything else!
pub const TEXT_CHARBLOCK: usize = 2;
/// Screenblock used by window 1
/// NOTE: Do not use for anything else!
pub const WINDOW_0_SCREENBLOCK: usize = 25;
/// Screenblock used by window 2
/// NOTE: Do not use for anything else!
pub const WINDOW_1_SCREENBLOCK: usize = 26;
/// The first background palette slot for use by the map.
pub const MAP_BG_PALETTE_START: usize = 0;
/// The last background palette slot for use by the map.
pub const MAP_BG_PALETTE_END: usize = 239;
/// The first background palette slot for use by the text engine.
pub const TEXT_BG_PALETTE_START: usize = 240;
/// The last background palette slot for use by the text engine.
pub const TEXT_BG_PALETTE_END: usize = 255;
