use crate::models::common::*;

/// 全てのシミュレーションエージェントが実装する基本インターフェース
pub trait IAgent {
    /// エージェントの初期化
    fn initialize(&mut self, scenario_config: &crate::scenario::ScenarioConfig);
    
    /// 1ティックの処理実行
    fn tick(&mut self, dt: f64);
    
    /// エージェントIDの取得
    fn get_id(&self) -> String;
    
    /// エージェントがアクティブかどうか
    fn is_active(&self) -> bool;
}

/// 移動可能なエージェントのインターフェース
pub trait IMovable {
    /// 移動処理
    fn move_agent(&mut self, dt: f64);
    
    /// 現在位置の取得
    fn get_position(&self) -> Position3D;
    
    /// 現在速度の取得
    fn get_velocity(&self) -> Velocity3D;
    
    /// 位置の設定
    fn set_position(&mut self, position: Position3D);
    
    /// 速度の設定
    fn set_velocity(&mut self, velocity: Velocity3D);
}

/// センサーのインターフェース
pub trait ISensor {
    /// ターゲットの探知
    fn detect(&mut self, targets: &[Box<dyn IAgent>]) -> Vec<String>;
    
    /// 探知範囲の取得
    fn get_detection_range(&self) -> f64;
    
    /// センサー位置の取得
    fn get_sensor_position(&self) -> Position3D;
}

/// プラットフォーム（ランチャー）のインターフェース
pub trait IPlatform {
    /// ミサイルの発射
    fn launch(&mut self, target_id: String) -> Option<Box<dyn IAgent>>;
    
    /// ミサイルのアサイン
    fn assign(&mut self, target_id: String) -> bool;
    
    /// 発射可能かどうか
    fn can_launch(&self) -> bool;
    
    /// 残弾数の取得
    fn get_remaining_missiles(&self) -> u32;
    
    /// クールダウン状態の取得
    fn get_cooldown_remaining(&self) -> f64;
}

/// ミサイルのインターフェース
pub trait IMissile {
    /// 誘導処理
    fn guidance(&mut self, target_position: Position3D, dt: f64);
    
    /// ターゲットIDの取得
    fn get_target_id(&self) -> String;
    
    /// 迎撃判定距離の取得
    fn get_intercept_radius(&self) -> f64;
}

/// 衝突検知のインターフェース
pub trait ICollision {
    /// 衝突判定
    fn check_collision(&self, target_position: Position3D) -> bool;
    
    /// miss distanceの計算
    fn calculate_miss_distance(&self, target_position: Position3D) -> f64;
    
    /// 終盤フェーズかどうかの判定
    fn is_endgame_phase(&self, target_position: Position3D) -> bool;
}

/// アロケーター（指揮所）のインターフェース
pub trait IAllocator {
    /// ターゲットの優先度付けとミサイル割り当て
    fn allocate(&mut self, detected_targets: &[String], launchers: &mut [Box<dyn IPlatform>]);
    
    /// 優先度の計算（Tgo計算）
    fn calculate_priority(&self, target_id: String) -> f64;
    
    /// ランチャーの選定
    fn select_launcher(&self, launchers: &[Box<dyn IPlatform>], target_position: Position3D) -> Option<usize>;
}