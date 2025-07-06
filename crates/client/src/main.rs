use glam::vec2;
use sdl3_sys::everything::*;
use std::error::Error;
use ui::{reactive::provide_context, Root};
use wz::Node;

mod app;
mod character;
mod cursor;
mod login;
mod map;
mod mob;
mod npc;
mod scene;
mod sound;
mod sprite;
mod timer;
mod widget;
mod wz;

#[derive(Clone)]
struct WzBase {
    pub node: Node,
}

fn main() -> Result<(), Box<dyn Error>> {
    async_runtime::block_on(run_app())
}

async fn run_app() -> Result<(), Box<dyn Error>> {

    unsafe {
        SDL_Init(SDL_INIT_VIDEO | SDL_INIT_AUDIO);
    }

    // let size = vec2(1920.0, 1080.0);
    // let size = vec2(1366.0, 768.0);
    // let size = unsafe {
    //     let mut num = MaybeUninit::<i32>::uninit();
    //     let displays = SDL_GetDisplays(num.as_mut_ptr());
    //     let mode = SDL_GetCurrentDisplayMode(*displays.offset(0));
    //     let w = (*mode).w;
    //     let h = (*mode).h;
    //     vec2(w as f32, h as f32)
    // };

    // let size = vec2(1600.0, 600.0);
    let size = vec2(800.0, 600.0);
    // let size = vec2(1024.0, 768.0);
    // let size = vec2(1366.0, 1024.0);

    let window = unsafe {
        SDL_CreateWindow(
            c"Maple RS".as_ptr(),
            size.x as i32,
            size.y as i32,
            // SDL_WINDOW_HIGH_PIXEL_DENSITY | SDL_WINDOW_BORDERLESS | SDL_WINDOW_MAXIMIZED | SDL_WINDOW_FULLSCREEN,
            SDL_WINDOW_HIGH_PIXEL_DENSITY,
        )
    };
    let renderer = unsafe { SDL_CreateRenderer(window, std::ptr::null()) };

    // Handle WZ data loading
    #[cfg(all(target_arch = "wasm32", target_os = "emscripten"))]
    {
        log::info!("Emscripten: Loading WZ data asynchronously");
        
        // Configure WZ loading with network support for all WZ files
        let mut config = wz::WzConfig::default();
        config.use_network = true;
        config.fallback_to_preload = true;
        
        // You can customize the base URL if needed
        // config.base_url = Some("https://your-server.com/Data".to_string());
        
        // Try to load WZ data asynchronously with network support
        match wz::resolve_base_with_config(&config).await {
            Ok(node) => {
                log::info!("Successfully loaded WZ data (including network resources)");
                // Debug: Check if Sound node exists
                if let Some(sound_node) = node.try_get("Sound") {
                    
                    // Check if Sound node has actual content
                    let sound_read = sound_node.wz_node.read().unwrap();
                    if sound_read.children.is_empty() {
                    }
                    drop(sound_read);
                    
                    // Try to access a known sound path
                    if let Ok(_test_sound) = node.at_path("Sound/UI.img/BtMouseOver") {
                        
                        // Try to see what's in Sound node
                        if let Ok(ui_node) = node.at_path("Sound/UI.img") {
                        }
                    }
                }
                
                provide_context(WzBase { node });
                Root::new(app::app, renderer).launch().await;
            }
            Err(e) => {
                log::error!("Failed to load WZ data: {}. Make sure to preload Base.wz with --preload-file or serve it via HTTP", e);
                // Create mock data and start anyway for testing
                use std::sync::{Arc, RwLock, Weak};
                use wz_parser::{WzNode, WzObjectType, property::WzSubProperty, WzNodeName};
                use indexmap::IndexMap;
                
                let mock_property = WzSubProperty::Property;
                let mock_node = WzNode {
                    name: WzNodeName::from("MockBase"),
                    object_type: WzObjectType::Property(mock_property),
                    parent: Weak::new(),
                    children: IndexMap::new(),
                };
                
                let wz_node = Arc::new(RwLock::new(mock_node));
                provide_context(WzBase {
                    node: wz_node.into(),
                });
                Root::new(app::app, renderer).launch().await;
            }
        }
        return Ok(());
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        let node = wz::resolve_base().await.unwrap();
        
        
        provide_context(WzBase { node });
        Root::new(app::app, renderer).launch().await;
        Ok(())
    }
}
