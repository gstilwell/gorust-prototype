#![cfg(target_arch = "wasm32")]

#[allow(unused_imports)]
use rg3d::{
    core::{
        algebra::{Matrix4, UnitQuaternion, Vector3},
        pool::Handle,
        color::Color,
        futures,
        wasm_bindgen::{self, prelude::*},
    },
    engine::{
        Engine,
        resource_manager::{ResourceManager, TextureImportOptions},
    },
    event::{DeviceEvent, ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    monitor::VideoMode,
    resource::texture::{CompressionOptions, TextureWrapMode},
    gui::{
        message::{MessageDirection, TextMessage},
        node::StubNode,
        text::TextBuilder,
        widget::WidgetBuilder,
    },
    dpi::{
        LogicalPosition,
        LogicalSize,
    },
    physics::{dynamics::RigidBodyBuilder, geometry::ColliderBuilder},
    scene::{
        graph::Graph,
        light::{BaseLightBuilder, PointLightBuilder},
        base::BaseBuilder,
        camera::{Camera, CameraBuilder, SkyBox},
        node::Node,
        transform::TransformBuilder,
        Scene,
        mesh::{
            surface::{SurfaceBuilder, SurfaceData},
            MeshBuilder,
        },
    },
    window::{
        WindowBuilder,
        Fullscreen,
    },
    utils::translate_event,
};

use std::{
    panic,
    sync::{Arc, Mutex, RwLock},
};

use serde_json::json;
use serde::{Serialize, Deserialize};

mod game_bits;

//use wasm_bindgen::prelude::*;

type UiNode = rg3d::gui::node::UINode<(), StubNode>;
type BuildContext<'a> = rg3d::gui::BuildContext<'a, (), StubNode>;

fn create_ui(ctx: &mut BuildContext) -> Handle<UiNode> {
    TextBuilder::new(WidgetBuilder::new()).build(ctx)
}

#[link(wasm_import_module = "../src/js/fullscreen.js")]
extern {fn addClickForFullscreen(); }

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn error(msg: String);

    type Error;

    #[wasm_bindgen(constructor)]
    fn new() -> Error;

    #[wasm_bindgen(structural, method, getter)]
    fn stack(error: &Error) -> String;
}

fn hook_impl(info: &panic::PanicInfo) {
    let mut msg = info.to_string();

    // Add the error stack to our message.
    //
    // This ensures that even if the `console` implementation doesn't
    // include stacks for `console.error`, the stack is still available
    // for the user. Additionally, Firefox's console tries to clean up
    // stack traces, and ruins Rust symbols in the process
    // (https://bugzilla.mozilla.org/show_bug.cgi?id=1519569) but since
    // it only touches the logged message's associated stack, and not
    // the message's contents, by including the stack in the message
    // contents we make sure it is available to the user.
    msg.push_str("\n\nStack:\n\n");
    let e = Error::new();
    let stack = e.stack();
    msg.push_str(&stack);

    // Safari's devtools, on the other hand, _do_ mess with logged
    // messages' contents, so we attempt to break their heuristics for
    // doing that by appending some whitespace.
    // https://github.com/rustwasm/console_error_panic_hook/issues/7
    msg.push_str("\n\n");

    // Finally, log the panic with `console.error`!
    error(msg);
}

/// A panic hook for use with
/// [`std::panic::set_hook`](https://doc.rust-lang.org/nightly/std/panic/fn.set_hook.html)
/// that logs panics into
/// [`console.error`](https://developer.mozilla.org/en-US/docs/Web/API/Console/error).
///
/// On non-wasm targets, prints the panic to `stderr`.
pub fn hook(info: &panic::PanicInfo) {
    hook_impl(info);
}

/// Set the `console.error` panic hook the first time this is called. Subsequent
/// invocations do nothing.
#[inline]
pub fn set_once() {
    use std::sync::Once;
    static SET_HOOK: Once = Once::new();
    SET_HOOK.call_once(|| {
        panic::set_hook(Box::new(hook));
    });
}

// Create our own engine type aliases. These specializations are needed, because the engine
// provides a way to extend UI with custom nodes and messages.
type GameEngine = Engine<(), StubNode>;

// Our game logic will be updated at 60 Hz rate.
const TIMESTEP: f32 = 1.0 / 60.0;

struct GameScene {
    scene: Scene,
}

struct SceneContext {
    data: Option<GameScene>,
}

/// Creates a camera at given position with a skybox.
pub async fn create_camera(
    resource_manager: ResourceManager,
    position: Vector3<f32>,
    graph: &mut Graph,
) -> Handle<Node> {
    // Load skybox textures in parallel.
    let (front, back, left, right, top, bottom) = rg3d::core::futures::join!(
        resource_manager.request_texture("assets/textures/DarkStormyFront.jpg"),
        resource_manager.request_texture("assets/textures/DarkStormyBack.jpg"),
        resource_manager.request_texture("assets/textures/DarkStormyLeft.jpg"),
        resource_manager.request_texture("assets/textures/DarkStormyRight.jpg"),
        resource_manager.request_texture("assets/textures/DarkStormyUp.jpg"),
        resource_manager.request_texture("assets/textures/DarkStormyDown.jpg")
    );

    // Unwrap everything.
    let skybox = SkyBox {
        front: Some(front.unwrap()),
        back: Some(back.unwrap()),
        left: Some(left.unwrap()),
        right: Some(right.unwrap()),
        top: Some(top.unwrap()),
        bottom: Some(bottom.unwrap()),
    };

    // Set S and T coordinate wrap mode, ClampToEdge will remove any possible seams on edges
    // of the skybox.
    for skybox_texture in skybox.textures().iter().filter_map(|t| t.clone()) {
        let mut data = skybox_texture.data_ref();
        data.set_s_wrap_mode(TextureWrapMode::ClampToEdge);
        data.set_t_wrap_mode(TextureWrapMode::ClampToEdge);
    }

    // Camera is our eyes in the world - you won't see anything without it.
    CameraBuilder::new(
        BaseBuilder::new().with_local_transform(
            TransformBuilder::new()
                .with_local_position(position)
                .build(),
        ),
    )
    .with_skybox(skybox)
    .build(graph)
}

async fn create_scene(resource_manager: ResourceManager, context: Arc<Mutex<SceneContext>>) {
    let mut scene = Scene::new();

    //let music = GenericSourceBuilder::new(
    //    resource_manager
    //        .request_sound_buffer("data/music.ogg", false)
    //        .await
    //        .unwrap()
    //        .into(),
    //)
    //.with_status(Status::Playing)
    //.build_source()
    //.unwrap();
    //
    //scene.sound_context.state().add_source(music);

    scene.ambient_lighting_color = Color::opaque(200, 200, 200);

    create_camera(
        resource_manager.clone(),
        Vector3::new(0.0, 6.0, -12.0),
        &mut scene.graph,
    )
    .await;

    PointLightBuilder::new(BaseLightBuilder::new(
        BaseBuilder::new().with_local_transform(
            TransformBuilder::new()
                .with_local_position(Vector3::new(0.0, 12.0, 0.0))
                .build(),
        ),
    ))
    .with_radius(20.0)
    .build(&mut scene.graph);

    //let (model_resource, walk_animation_resource) = rg3d::core::futures::join!(
    //    resource_manager.request_model("data/mutant.FBX"),
    //    resource_manager.request_model("data/walk.fbx")
    //);

    //// Instantiate model on scene - but only geometry, without any animations.
    //// Instantiation is a process of embedding model resource data in desired scene.
    //let model = model_resource.unwrap().instantiate_geometry(&mut scene);

    //// Now we have whole sub-graph instantiated, we can start modifying model instance.
    //scene.graph[model]
    //    .local_transform_mut()
    //    // Our model is too big, fix it by scale.
    //    .set_scale(Vector3::new(0.05, 0.05, 0.05));

    //// Add simple animation for our model. Animations are loaded from model resources -
    //// this is because animation is a set of skeleton bones with their own transforms.
    //// Once animation resource is loaded it must be re-targeted to our model instance.
    //// Why? Because animation in *resource* uses information about *resource* bones,
    //// not model instance bones, retarget_animations maps animations of each bone on
    //// model instance so animation will know about nodes it should operate on.
    //let walk_animation = *walk_animation_resource
    //    .unwrap()
    //    .retarget_animations(model, &mut scene)
    //    .get(0)
    //    .unwrap();

    // Add floor.
    MeshBuilder::new(
        BaseBuilder::new().with_local_transform(
            TransformBuilder::new()
                .with_local_position(Vector3::new(0.0, -0.25, 0.0))
                .build(),
        ),
    )
    .with_surfaces(vec![SurfaceBuilder::new(Arc::new(RwLock::new(
        SurfaceData::make_cube(Matrix4::new_nonuniform_scaling(&Vector3::new(
            25.0, 0.25, 25.0,
        ))),
    )))
    .with_diffuse_texture(resource_manager.request_texture("assets/textures/floor.jpg"))
    .build()])
    .build(&mut scene.graph);

    context.lock().unwrap().data = Some(GameScene {
        scene,
    })
}


//impl Game {
//    pub async fn new(engine: &mut GameEngine) -> Self {
//        let mut scene = Scene::new();
//
//        // Load a scene resource and create its instance.
//        //rg3d::core::wasm_bindgen_futures::spawn_local(
//        //    engine.resource_manager.request_model("assets/models/my_favorite_scene.rgs")
//        //);
//                //.await
//                //.unwrap()
//                //.instantiate_geometry(&mut scene);
//
//        //// Next create a camera, it is our "eyes" in the world.
//        //// This can also be made in editor, but for educational purpose we'll made it by hand.
//        //let camera = CameraBuilder::new(
//        //    BaseBuilder::new().with_local_transform(
//        //        TransformBuilder::new()
//        //            .with_local_position(Vector3::new(0.0, 1.0, -3.0))
//        //            .build(),
//        //    ),
//        //)
//        //.build(&mut scene.graph);
//
//        //Self {
//        //    camera: camera,
//        //    scene: engine.scenes.add(scene),
//        //}
//        Self {}
//    }
//
//    pub fn update(&mut self) {
//        // Game logic will be placed here.
//    }
//}

struct ScreenSize {
    width: u32,
    height: u32,
}

#[wasm_bindgen]
pub fn main() {
    set_once();

    let mut pointy = LogicalPosition {
        x: 0.0,
        y: 0.0,
    };

    let mut ws = game_bits::websocket::Websocket{ ws: None, client_id: 0 };
    ws.start();

    // Configure main window first.
    let window_builder = WindowBuilder::new().with_title("Gorust!");
    // Create event loop that will be used to "listen" events from the OS.
    let event_loop = EventLoop::new();

    // Finally create an instance of the engine.
    let mut engine = GameEngine::new(window_builder, &event_loop, true).unwrap();
    engine.renderer.set_backbuffer_clear_color(Color::opaque(150, 150, 255));

    // Configure resource manager.
    engine.resource_manager.state().set_textures_import_options(
        TextureImportOptions::default().with_compression(CompressionOptions::NoCompression),
    );
    engine
        .resource_manager
        .state()
        .set_textures_path("assets/textures");

    let mut screen_size = ScreenSize {
        width: engine.get_window().inner_size().width,
        height: engine.get_window().inner_size().height,
    };

    // Initialize game instance. It is empty for now.
    //let mut game = futures::executor::block_on(Game::new(&mut engine));
    let load_context = Arc::new(Mutex::new(SceneContext { data: None }));
    rg3d::core::wasm_bindgen_futures::spawn_local(create_scene(engine.resource_manager.clone(), load_context.clone()));

    let mut scene_handle = Handle::NONE;

    let debug_text = create_ui(&mut engine.user_interface.build_ctx());

    unsafe {
        addClickForFullscreen();
    }
    // Run the event loop of the main window. which will respond to OS and window events and update
    // engine's state accordingly. Engine lets you to decide which event should be handled,
    // this is minimal working example if how it should be.
    let clock = rg3d::core::instant::Instant::now();

    let mut elapsed_time = 0.0;
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::MainEventsCleared => {
                // This main game loop - it has fixed time step which means that game
                // code will run at fixed speed even if renderer can't give you desired
                // 60 fps.
                let mut dt = clock.elapsed().as_secs_f32() - elapsed_time;
                while dt >= TIMESTEP {
                    dt -= TIMESTEP;
                    elapsed_time += TIMESTEP;

                    // Run our game's logic.
                    //game.update();
                    if let Some(scene) = load_context.lock().unwrap().data.take() {
                        scene_handle = engine.scenes.add(scene.scene);
                    }

                    if scene_handle.is_some() {
                        let scene = &mut engine.scenes[scene_handle];
                    }

                    let _fps = engine.renderer.get_statistics().frames_per_second;
                    let text = format!(
                        "Click for full screen\nscreen size: {}, {}\npointy: {}, {}",
                        screen_size.width, screen_size.height,
                        pointy.x, pointy.y
                    );
                    engine.user_interface.send_message(TextMessage::text(
                        debug_text,
                        MessageDirection::ToWidget,
                        text,
                    ));

                    // Update engine each frame.
                    engine.update(TIMESTEP);
                }

                // It is very important to "pump" messages from UI. Even if don't need to
                // respond to such message, you should call this method, otherwise UI
                // might behave very weird.
                while let Some(_ui_event) = engine.user_interface.poll_message() {
                    // ************************
                    // Put your data model synchronization code here. It should
                    // take message and update data in your game according to
                    // changes in UI.
                    // ************************
                }

                // Rendering must be explicitly requested and handled after RedrawRequested event is received.
                engine.get_window().request_redraw();
            }
            Event::RedrawRequested(_) => {
                // Render at max speed - it is not tied to the game code.
                engine.render(TIMESTEP).unwrap();
            }
            Event::DeviceEvent { event, .. } => {
                match event {
                    DeviceEvent::MouseMotion { delta } => {
                        pointy.x += delta.0;
                        pointy.y += delta.1;

                        #[derive(Serialize, Deserialize)]
                        struct Payload<'a>{
                            messageType: &'a str,
                            clientId: u32,
                            x: f64,
                            y: f64,
                        };
                        ws.send_message(Payload{
                            messageType: "cursorPosition",
                            clientId: ws.client_id,
                            x: pointy.x,
                            y: pointy.y,
                        });
                    },
                    _ => (),
                }
            }
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit
                    },
                    WindowEvent::KeyboardInput { input, .. } => {
                        // Exit game by hitting Escape.
                        if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
                            *control_flow = ControlFlow::Exit
                        }
                    },
			        WindowEvent::Resized(size) => {
                        // It is very important to handle Resized event from window, because
                        // renderer knows nothing about window size - it must be notified
                        // directly when window size has changed.
                        screen_size.width = size.width;
                        screen_size.height = size.height;
                        engine.renderer.set_frame_size((screen_size.width, screen_size.height));
                    },
                    _ => (),
                }

                // It is very important to "feed" user interface (UI) with events coming
                // from main window, otherwise UI won't respond to mouse, keyboard, or any
                // other event.
                if let Some(os_event) = translate_event(&event) {
                    engine.user_interface.process_os_event(&os_event);
                }
            }
            
            _ => *control_flow = ControlFlow::Poll,
        }

    });
}