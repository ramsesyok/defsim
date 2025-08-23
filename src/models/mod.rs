// 基本的なデータ型と数学ユーティリティ
pub mod common;

// エージェントの基本インターフェース（trait）定義
pub mod traits;

// 各エージェントモデルの実装
pub mod target;
pub mod command_post;
pub mod sensor;
pub mod launcher;
pub mod missile;

// 便利な re-export
pub use common::*;
pub use traits::*;
pub use target::{Target, TargetGroup};
pub use command_post::{CommandPost, TargetPriority};
pub use sensor::{Sensor, SensorNetwork, DetectionEvent, DetectionEventType, DetectionStats};
pub use launcher::{Launcher, LauncherBattery, LaunchRecord, LaunchStats, BatteryStats};
pub use missile::{Missile, GuidancePhase, MissileEndReason, Attitude3D};