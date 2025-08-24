//! # Models モジュール
//! 
//! 防衛シミュレーションシステムのエージェントモデルとデータ型を定義します。
//! 
//! このモジュールは、シミュレーション内で動作するすべてのエージェント（ターゲット、
//! 指揮所、センサー、ランチャー、ミサイル）の実装と、それらが共通して使用する
//! データ型、インターフェース、数学ユーティリティを提供します。
//! 
//! ## 主要コンポーネント
//! 
//! - **common**: 3次元座標、速度、加速度などの基本データ型と数学ユーティリティ
//! - **traits**: 全エージェントが実装すべき共通インターフェースの定義
//! - **target**: 敵ターゲットエージェントとグループ配置機能
//! - **command_post**: 中央指揮所エージェントとターゲット優先度管理
//! - **sensor**: ターゲット検知センサーエージェントとネットワーク機能
//! - **launcher**: ミサイル発射ランチャーエージェントと統計機能
//! - **missile**: 誘導ミサイルエージェントと3次元誘導アルゴリズム
//! 
//! ## エージェントアーキテクチャ
//! 
//! すべてのエージェントは`IAgent`トレイトを実装し、共通のライフサイクル
//! （初期化、時間更新、状態管理）を持ちます。移動可能なエージェントは
//! さらに`IMovable`トレイトを実装し、各エージェントタイプ固有の機能は
//! 専用のトレイトで定義されています。
//! 
//! ## 座標系と単位
//! 
//! - 座標系: X軸（右方向）、Y軸（上方向）、Z軸（高度）
//! - 距離: メートル（m）
//! - 時間: 秒（s）
//! - 角度: 度（°）（外部インターフェース）、ラジアン（内部計算）

/// 基本的なデータ型と数学ユーティリティ
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