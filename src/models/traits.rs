use crate::models::common::*;

/// 全てのシミュレーションエージェントが実装する基本インターフェース
/// 
/// すべてのシミュレーションエージェントが共通して持つ必要がある機能を定義します。
/// エージェントのライフサイクル管理、時間更新、状態管理を担います。
pub trait IAgent {
    /// エージェントの初期化
    /// 
    /// シナリオ設定を基にエージェントのパラメータを設定し、初期状態を準備します。
    /// 
    /// # 引数
    /// 
    /// * `scenario_config` - シナリオ設定情報
    fn initialize(&mut self, scenario_config: &crate::scenario::ScenarioConfig);
    
    /// 1ティックの処理実行
    /// 
    /// シミュレーションの各タイムステップで呼び出され、エージェントの状態を更新します。
    /// 
    /// # 引数
    /// 
    /// * `dt` - 時間ステップ（秒）
    fn tick(&mut self, dt: f64);
    
    /// エージェントIDの取得
    /// 
    /// # 戻り値
    /// 
    /// エージェントの一意識別子
    fn get_id(&self) -> String;
    
    /// エージェントがアクティブかどうか
    /// 
    /// # 戻り値
    /// 
    /// エージェントがアクティブ状態の場合true
    fn is_active(&self) -> bool;
}

/// 移動可能なエージェントのインターフェース
/// 
/// 位置や速度を持ち、空間内で移動するエージェントが実装すべきインターフェースです。
/// TargetやMissileなどの移動するエージェントが実装します。
pub trait IMovable {
    /// 移動処理
    /// 
    /// 指定された時間ステップに基づいてエージェントの位置を更新します。
    /// 
    /// # 引数
    /// 
    /// * `dt` - 時間ステップ（秒）
    fn move_agent(&mut self, dt: f64);
    
    /// 現在位置の取得
    /// 
    /// # 戻り値
    /// 
    /// エージェントの現在の3次元位置
    fn get_position(&self) -> Position3D;
    
    /// 現在速度の取得
    /// 
    /// # 戻り値
    /// 
    /// エージェントの現在の3次元速度ベクトル
    fn get_velocity(&self) -> Velocity3D;
    
    /// 位置の設定
    /// 
    /// # 引数
    /// 
    /// * `position` - 設定する新しい位置
    fn set_position(&mut self, position: Position3D);
    
    /// 速度の設定
    /// 
    /// # 引数
    /// 
    /// * `velocity` - 設定する新しい速度ベクトル
    fn set_velocity(&mut self, velocity: Velocity3D);
}

/// センサーのインターフェース
/// 
/// 敵ターゲットを検知し、指揮所に情報を提供するセンサーシステムが実装すべきインターフェースです。
pub trait ISensor {
    /// ターゲットの探知
    /// 
    /// 指定されたエージェントの中からセンサーの検知範囲内にあるターゲットを検知します。
    /// 
    /// # 引数
    /// 
    /// * `targets` - 検知対象のエージェントリスト
    /// 
    /// # 戻り値
    /// 
    /// 検知されたターゲットIDのベクター
    fn detect(&mut self, targets: &[Box<dyn IAgent>]) -> Vec<String>;
    
    /// 探知範囲の取得
    /// 
    /// # 戻り値
    /// 
    /// センサーの検知範囲（メートル）
    fn get_detection_range(&self) -> f64;
    
    /// センサー位置の取得
    /// 
    /// # 戻り値
    /// 
    /// センサーの現在位置
    fn get_sensor_position(&self) -> Position3D;
}

/// プラットフォーム（ランチャー）のインターフェース
/// 
/// ミサイルを発射し、ターゲットに対する攻撃を行うランチャーシステムが実装すべきインターフェースです。
pub trait IPlatform {
    /// ミサイルの発射
    /// 
    /// 指定されたターゲットに向けてミサイルを発射します。
    /// 
    /// # 引数
    /// 
    /// * `target_id` - 攻撃対象のターゲットID
    /// 
    /// # 戻り値
    /// 
    /// 発射されたミサイルエージェント、発射不可の場合はNone
    fn launch(&mut self, target_id: String) -> Option<Box<dyn IAgent>>;
    
    /// ミサイルのアサイン
    /// 
    /// ターゲットを発射キューにアサインし、将来の発射に備えます。
    /// 
    /// # 引数
    /// 
    /// * `target_id` - アサインするターゲットID
    /// 
    /// # 戻り値
    /// 
    /// アサインに成功した場合true
    fn assign(&mut self, target_id: String) -> bool;
    
    /// 発射可能かどうか
    /// 
    /// # 戻り値
    /// 
    /// ランチャーが発射準備完了状態の場合true
    fn can_launch(&self) -> bool;
    
    /// 残弾数の取得
    /// 
    /// # 戻り値
    /// 
    /// 現在装備しているミサイル数
    fn get_remaining_missiles(&self) -> u32;
    
    /// クールダウン状態の取得
    /// 
    /// # 戻り値
    /// 
    /// 残りクールダウン時間（秒）
    fn get_cooldown_remaining(&self) -> f64;
}

/// ミサイルのインターフェース
/// 
/// ターゲットに向かって誘導されるミサイルが実装すべきインターフェースです。
pub trait IMissile {
    /// 誘導処理
    /// 
    /// ターゲット位置に基づいてミサイルの誘導と運動を更新します。
    /// 
    /// # 引数
    /// 
    /// * `target_position` - ターゲットの現在位置
    /// * `dt` - 時間ステップ（秒）
    fn guidance(&mut self, target_position: Position3D, dt: f64);
    
    /// ターゲットIDの取得
    /// 
    /// # 戻り値
    /// 
    /// このミサイルが攻撃中のターゲットID
    fn get_target_id(&self) -> String;
    
    /// 迎撃判定距離の取得
    /// 
    /// # 戻り値
    /// 
    /// ミサイルの迎撃判定距離（メートル）
    fn get_intercept_radius(&self) -> f64;
}

/// 衝突検知のインターフェース
/// 
/// ミサイルとターゲット間の衝突判定やmiss distanceの計算を行うインターフェースです。
pub trait ICollision {
    /// 衝突判定
    /// 
    /// ミサイルとターゲットが衝突（迎撃）したかを判定します。
    /// 
    /// # 引数
    /// 
    /// * `target_position` - ターゲットの位置
    /// 
    /// # 戻り値
    /// 
    /// 衝突した場合true
    fn check_collision(&self, target_position: Position3D) -> bool;
    
    /// miss distanceの計算
    /// 
    /// ミサイルとターゲット間の最短距離を計算します。
    /// 
    /// # 引数
    /// 
    /// * `target_position` - ターゲットの位置
    /// 
    /// # 戻り値
    /// 
    /// miss distance（メートル）
    fn calculate_miss_distance(&self, target_position: Position3D) -> f64;
    
    /// 終盤フェーズかどうかの判定
    /// 
    /// ミサイルがターゲットに近づき、終盤誘導フェーズに入ったかを判定します。
    /// 
    /// # 引数
    /// 
    /// * `target_position` - ターゲットの位置
    /// 
    /// # 戻り値
    /// 
    /// 終盤フェーズの場合true
    fn is_endgame_phase(&self, target_position: Position3D) -> bool;
}

/// アロケーター（指揮所）のインターフェース
/// 
/// 検知されたターゲットの優先度を評価し、適切なランチャーへミサイル割り当てを行う
/// 中央統制システム（指揮所）が実装すべきインターフェースです。
pub trait IAllocator {
    /// ターゲットの優先度付けとミサイル割り当て
    /// 
    /// 検知されたターゲットを評価し、優先度に基づいて適切なランチャーにミサイルを割り当てます。
    /// 
    /// # 引数
    /// 
    /// * `detected_targets` - 検知されたターゲットIDのリスト
    /// * `launchers` - 利用可能なランチャーの可変スライス
    fn allocate(&mut self, detected_targets: &[String], launchers: &mut [Box<dyn IPlatform>]);
    
    /// 優先度の計算（Tgo計算）
    /// 
    /// 指定されたターゲットの脅威度を評価し、優先度値を計算します。
    /// 
    /// # 引数
    /// 
    /// * `target_id` - 評価するターゲットID
    /// 
    /// # 戻り値
    /// 
    /// 優先度値（低いほど高優先）
    fn calculate_priority(&self, target_id: String) -> f64;
    
    /// ランチャーの選定
    /// 
    /// 指定されたターゲット位置に対して最適なランチャーを選定します。
    /// 
    /// # 引数
    /// 
    /// * `launchers` - 選定対象のランチャースライス
    /// * `target_position` - ターゲットの位置
    /// 
    /// # 戻り値
    /// 
    /// 選定されたランチャーのインデックス、選定不可の場合はNone
    fn select_launcher(&self, launchers: &[Box<dyn IPlatform>], target_position: Position3D) -> Option<usize>;
}