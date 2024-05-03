//#![windows_subsystem = "windows"]
use bevy::{input::mouse::*, prelude::*, window::*};
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin};
use bevy_screen_diagnostics::{ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin};
mod initialize;
fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bevy".into(),                                          //タイトル
                        resolution: (1280.0, 720.0).into(), //ウィンドウサイズ
                        position: WindowPosition::Centered(MonitorSelection::Primary), //ウィンドウの生成座標を中心に設定
                        present_mode: PresentMode::AutoNoVsync, //Vsyncを無効化
                        resizable: false,                       //サイズ変更不可
                        enabled_buttons: bevy::window::EnabledButtons {
                            minimize: false, //最小化無効
                            maximize: false, //最大化無効
                            close: true,     //閉じる有効
                        },
                        visible: false, //非表示
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()), //デフォルトの画像処理をピクセルパーフェクトに設定
        )
        .init_gizmo_group::<MyRoundGizmos>()
        .insert_resource(ClearColor(Color::NONE)) //デフォルトの背景色を設定
        .insert_resource(Msaa::Off) //MSAAを無効化
        .add_plugins((
            initialize::InitializePlugin,       //初期処理
            ScreenDiagnosticsPlugin::default(), //診断情報を表示
            ScreenFrameDiagnosticsPlugin,       //フレームレートを表示
            InfiniteGridPlugin,                 //グリッドを表示
        ))
        //以上は固定
        .add_systems(Startup, (summon, set_world))
        .add_systems(
            Update,
            (
                move_player,
                move_camera,
                mouse_move,
                render_gizmos,
                update_gizmos,
            ),
        )
        .run();
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct DebugObject;

#[derive(Default, Reflect, GizmoConfigGroup)]
struct MyRoundGizmos {}

pub fn set_world(mut commands: Commands) {
    commands.spawn(InfiniteGridBundle::default());

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

pub fn summon(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Player
    let default_fov = 70.53_f32.to_radians(); //デフォルトの垂直視野角70.53度(水平視野角は103度)
    commands.spawn((
        Player,
        DebugObject,
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(Color::rgb_u8(124, 144, 255)),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
    ));

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(4.0, 4.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
            projection: Projection::Perspective(PerspectiveProjection {
                fov: default_fov,
                ..default()
            }),
            ..default()
        },
        FogSettings {
            color: Color::rgba(0., 0., 0., 1.0),
            falloff: FogFalloff::Linear {
                start: 20.0,
                end: 48.0,
            },
            ..default()
        },
        SpatialListener {
            left_ear_offset: Vec3::new(0.1, 0.0, 0.0),
            right_ear_offset: Vec3::new(-0.1, 0.0, 0.0),
        },
    ));
}

fn move_player(
    mut query: Query<&mut Transform, With<Player>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let speed = 12.;
    for mut transform in query.iter_mut() {
        let rotation = transform.rotation;
        if keyboard_input.pressed(KeyCode::KeyW) {
            transform.translation -= rotation * Vec3::Z * speed * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            transform.translation += rotation * Vec3::Z * speed * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            transform.translation -= rotation * Vec3::X * speed * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            transform.translation += rotation * Vec3::X * speed * time.delta_seconds();
        }
    }
}

fn move_camera(
    mut query: Query<&mut Transform, With<Player>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let sensitivity = 2.;
    for mut transform in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::ArrowRight) {
            transform.rotation *= Quat::from_rotation_y(-sensitivity * time.delta_seconds());
        }
        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            transform.rotation *= Quat::from_rotation_y(sensitivity * time.delta_seconds());
        }
    }
}

fn mouse_move(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    for event in mouse_motion_events.read() {
        for mut transform in query.iter_mut() {
            let sensitivity = 1.;
            let delta = Vec2::new(event.delta.y, event.delta.x) * sensitivity;
            transform.rotation *= Quat::from_rotation_x(-delta.x * time.delta_seconds());
            transform.rotation *= Quat::from_rotation_y(-delta.y * time.delta_seconds());
        }
    }
}

fn render_gizmos(
    mut gizmos: Gizmos<MyRoundGizmos>,
    mut query: Query<&mut Transform, With<DebugObject>>,
) {
    for transform in query.iter_mut() {
        gizmos.arrow(
            transform.translation,
            transform.translation + transform.rotation * Vec3::Z * -2.0,
            Color::YELLOW,
        );
        gizmos.ray(
            transform.translation,
            transform.rotation * Vec3::X,
            Color::RED,
        );
        gizmos.ray(
            transform.translation,
            transform.rotation * Vec3::Y,
            Color::GREEN,
        );
        gizmos.ray(
            transform.translation,
            transform.rotation * Vec3::Z,
            Color::BLUE,
        );
    }
}

fn update_gizmos(mut config_store: ResMut<GizmoConfigStore>) {
    for (_, config, _) in config_store.iter_mut() {
        config.depth_bias = -1.;
    }
}
