use std::collections::{HashSet, HashMap};
use crate::models::{
    traits::{IAgent, ISensor},
    common::{Position3D, AgentStatus},
    target::Target,
};

/// センサーエージェント
/// 
/// 敵ターゲットを検知し、指揮所に情報を提供するセンサーシステムです。
/// 球形の検知範囲を持ち、ターゲットの初回検知、追跡、ロストを管理します。
#[derive(Debug, Clone)]
pub struct Sensor {
    /// センサーの一意識別子
    pub id: String,
    /// センサーの3次元位置
    pub position: Position3D,
    /// 探知範囲（メートル、球形半径）
    pub detection_range: f64,
    /// センサーの現在状態
    pub status: AgentStatus,
    /// 現在検知中のターゲットIDセット
    pub detected_targets: HashSet<String>,
    /// 検知イベントの履歴
    pub detection_history: Vec<DetectionEvent>,
}

/// 検知イベント
/// 
/// センサーがターゲットを検知、追跡、またはロストしたことを記録します。
#[derive(Debug, Clone)]
pub struct DetectionEvent {
    /// イベント発生時刻（シミュレーション開始からの経過秒数）
    pub timestamp: f64,
    /// ターゲットのID
    pub target_id: String,
    /// イベント発生時のターゲット位置
    pub target_position: Position3D,
    /// センサーからターゲットまでの距離（メートル）
    pub distance: f64,
    /// 検知イベントの種類
    pub event_type: DetectionEventType,
}

/// 検知イベントの種類
/// 
/// センサーがターゲットに対して行ったアクションの種類を表します。
#[derive(Debug, Clone, PartialEq)]
pub enum DetectionEventType {
    /// ターゲットを初めて検知した
    FirstDetected,
    /// ターゲットを追跡中
    Tracking,
    /// ターゲットをロストした
    Lost,
}

impl Sensor {
    /// 新しいセンサーを作成します
    /// 
    /// # 引数
    /// 
    /// * `id` - センサーの一意識別子
    /// * `position` - センサーの3次元位置
    /// 
    /// # 戻り値
    /// 
    /// 初期化されたセンサーインスタンス（initializeメソッドで詳細設定が必要）
    pub fn new(id: String, position: Position3D) -> Self {
        Self {
            id,
            position,
            detection_range: 0.0,               // initializeで設定
            status: AgentStatus::Active,
            detected_targets: HashSet::new(),
            detection_history: Vec::new(),
        }
    }

    /// ターゲットの検知処理
    /// 
    /// 指定されたターゲットリストに対して検知処理を実行し、
    /// 初回検知、追跡、ロストのイベントを記録します。
    /// 
    /// # 引数
    /// 
    /// * `targets` - 検知対象のターゲットスライス
    /// * `current_time` - 現在のシミュレーション時刻（秒）
    /// 
    /// # 戻り値
    /// 
    /// 現在検知中のターゲットIDのベクター
    pub fn detect_targets(&mut self, targets: &[Target], current_time: f64) -> Vec<String> {
        let mut newly_detected = Vec::new();
        let mut currently_detected = HashSet::new();

        for target in targets {
            if !target.is_active() {
                continue;
            }

            let distance = self.position.distance_3d(&target.position);
            
            if distance <= self.detection_range {
                currently_detected.insert(target.id.clone());
                
                // 初回検知かどうか
                let is_newly_detected = !self.detected_targets.contains(&target.id);
                
                if is_newly_detected {
                    newly_detected.push(target.id.clone());
                    
                    // 検知イベントを記録
                    self.detection_history.push(DetectionEvent {
                        timestamp: current_time,
                        target_id: target.id.clone(),
                        target_position: target.position,
                        distance,
                        event_type: DetectionEventType::FirstDetected,
                    });
                } else {
                    // 追跡中イベントを記録
                    self.detection_history.push(DetectionEvent {
                        timestamp: current_time,
                        target_id: target.id.clone(),
                        target_position: target.position,
                        distance,
                        event_type: DetectionEventType::Tracking,
                    });
                }
            }
        }

        // ロストしたターゲットの処理
        for target_id in &self.detected_targets {
            if !currently_detected.contains(target_id) {
                // ロストイベントを記録
                self.detection_history.push(DetectionEvent {
                    timestamp: current_time,
                    target_id: target_id.clone(),
                    target_position: Position3D::new(0.0, 0.0, 0.0), // 不明
                    distance: 0.0,
                    event_type: DetectionEventType::Lost,
                });
            }
        }

        // 検知状態を更新
        self.detected_targets = currently_detected.clone();
        
        // 現在検知中の全ターゲットIDを返す
        currently_detected.into_iter().collect()
    }

    /// 特定のターゲットとの距離を計算
    /// 
    /// # 引数
    /// 
    /// * `target` - 距離を測定するターゲット
    /// 
    /// # 戻り値
    /// 
    /// 3次元空間での距離（メートル）
    pub fn distance_to_target(&self, target: &Target) -> f64 {
        self.position.distance_3d(&target.position)
    }

    /// センサーの有効性チェック
    /// 
    /// # 戻り値
    /// 
    /// センサーがアクティブで動作中の場合はtrue
    pub fn is_operational(&self) -> bool {
        self.status == AgentStatus::Active
    }

    /// 検知履歴の取得（最新N件）
    /// 
    /// # 引数
    /// 
    /// * `count` - 取得する最新イベントの数
    /// 
    /// # 戻り値
    /// 
    /// 最新の検知イベントの参照ベクター
    pub fn get_recent_detections(&self, count: usize) -> Vec<&DetectionEvent> {
        let start_index = if self.detection_history.len() > count {
            self.detection_history.len() - count
        } else {
            0
        };
        
        self.detection_history[start_index..].iter().collect()
    }

    /// 検知統計の計算
    /// 
    /// # 戻り値
    /// 
    /// センサーの検知統計情報を含むDetectionStats構造体
    pub fn get_detection_stats(&self) -> DetectionStats {
        let total_detections = self.detection_history.len();
        let first_detections = self.detection_history
            .iter()
            .filter(|event| event.event_type == DetectionEventType::FirstDetected)
            .count();
        let lost_detections = self.detection_history
            .iter()
            .filter(|event| event.event_type == DetectionEventType::Lost)
            .count();
        let currently_tracking = self.detected_targets.len();

        DetectionStats {
            total_detections,
            first_detections,
            lost_detections,
            currently_tracking,
        }
    }

    /// 検知履歴をクリア（メモリ管理用）
    /// 
    /// 累積された検知履歴をすべて削除し、メモリ使用量をリセットします。
    pub fn clear_detection_history(&mut self) {
        self.detection_history.clear();
    }


    /// 検知済みターゲットIDのリストを取得
    /// 
    /// # 戻り値
    /// 
    /// 現在検知中のターゲットIDのベクター
    pub fn get_detected_targets(&self) -> Vec<String> {
        self.detected_targets.iter().cloned().collect()
    }

    /// 検知範囲内かどうかの判定
    /// 
    /// 指定された位置がセンサーの検知範囲内にあるかを判定します。
    /// 
    /// # 引数
    /// 
    /// * `position` - 判定する3次元位置
    /// 
    /// # 戻り値
    /// 
    /// 検知範囲内にある場合はtrue
    pub fn is_in_detection_range(&self, position: Position3D) -> bool {
        self.position.distance_3d(&position) <= self.detection_range
    }

    /// ターゲット検知の更新（シミュレーションエンジン用）
    /// 
    /// シミュレーションエンジンから呼び出されるラッパーメソッドで、
    /// detect_targetsメソッドを呼び出して検知処理を行います。
    /// 
    /// # 引数
    /// 
    /// * `targets` - 検知対象のターゲットスライス
    /// * `current_time` - 現在のシミュレーション時刻（秒）
    pub fn update_detections(&mut self, targets: &[Target], current_time: f64) {
        self.detect_targets(targets, current_time);
    }
}

/// 検知統計情報
/// 
/// センサーの検知性能と現在の状態を表す統計情報です。
#[derive(Debug, Clone)]
pub struct DetectionStats {
    /// 総検知イベント数
    pub total_detections: usize,
    /// 初回検知イベント数
    pub first_detections: usize,
    /// ロストイベント数
    pub lost_detections: usize,
    /// 現在追跡中のターゲット数
    pub currently_tracking: usize,
}

impl IAgent for Sensor {
    fn initialize(&mut self, scenario_config: &crate::scenario::ScenarioConfig) {
        self.status = AgentStatus::Active;
        self.detected_targets.clear();
        self.detection_history.clear();
        
        // シナリオからセンサー設定を探して適用
        for sensor_config in &scenario_config.friendly_forces.sensors {
            if sensor_config.id == self.id {
                self.detection_range = sensor_config.range_m;
                break;
            }
        }
        
        // 距離測定方式の設定
        let distance_convention = &scenario_config.world.distance_conventions.sensor;
        match distance_convention.as_str() {
            "3D" => {
                // 3D距離での検知（デフォルト）
            },
            "XY" => {
                // XY平面距離での検知
            },
            _ => {
                // デフォルト
            }
        }
    }

    fn tick(&mut self, dt: f64) {
        // センサーは基本的に受動的なデバイス
        // 実際の検知処理は外部から detect_targets が呼ばれることで実行される
        
        // 必要に応じて、ここでセンサーの自己診断や状態監視を実装
        if !self.is_operational() {
            return;
        }
        
        // 古い検知履歴の削除（メモリ管理）
        static mut CURRENT_TIME: f64 = 0.0;
        unsafe {
            CURRENT_TIME += dt;
            
            // 60秒より古い履歴は削除
            let cutoff_time = CURRENT_TIME - 60.0;
            self.detection_history.retain(|event| event.timestamp >= cutoff_time);
        }
    }

    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn is_active(&self) -> bool {
        self.status == AgentStatus::Active
    }
}

impl ISensor for Sensor {
    fn detect(&mut self, _targets: &[Box<dyn IAgent>]) -> Vec<String> {
        // この実装では Target 固有のメソッドが必要なため、
        // 実際のシミュレーションでは detect_targets メソッドを使用することを想定
        
        // プレースホルダー実装
        Vec::new()
    }

    fn get_detection_range(&self) -> f64 {
        self.detection_range
    }

    fn get_sensor_position(&self) -> Position3D {
        self.position
    }
}

/// センサーネットワーク管理用のヘルパー構造体
/// 
/// 複数のセンサーを一括管理し、ネットワーク全体での検知処理や
/// データ融合、エリアカバレッジ解析を提供します。
#[derive(Debug)]
pub struct SensorNetwork {
    /// ネットワークに所属するセンサーのリスト
    pub sensors: Vec<Sensor>,
    /// データ融合機能の有効/無効
    pub fusion_enabled: bool,
}

impl SensorNetwork {
    /// 新しいセンサーネットワークを作成
    /// 
    /// # 戻り値
    /// 
    /// データ融合機能が有効な空のセンサーネットワーク
    pub fn new() -> Self {
        Self {
            sensors: Vec::new(),
            fusion_enabled: true,
        }
    }

    /// ネットワークにセンサーを追加
    /// 
    /// # 引数
    /// 
    /// * `sensor` - 追加するセンサー
    pub fn add_sensor(&mut self, sensor: Sensor) {
        self.sensors.push(sensor);
    }

    /// ネットワーク全体でのターゲット検知
    /// 
    /// ネットワーク内のすべてのオペレーショナルなセンサーで検知を実行し、
    /// データ融合が有効な場合は重複を排除した結果を返します。
    /// 
    /// # 引数
    /// 
    /// * `targets` - 検知対象のターゲットスライス
    /// * `current_time` - 現在のシミュレーション時刻（秒）
    /// 
    /// # 戻り値
    /// 
    /// ネットワーク全体で検知されたターゲットIDのベクター
    pub fn network_detect(&mut self, targets: &[Target], current_time: f64) -> Vec<String> {
        let mut all_detected = HashSet::new();

        for sensor in &mut self.sensors {
            if sensor.is_operational() {
                let detected = sensor.detect_targets(targets, current_time);
                all_detected.extend(detected);
            }
        }

        // データ融合処理（重複排除）
        if self.fusion_enabled {
            all_detected.into_iter().collect()
        } else {
            // 各センサーの検知結果をそのまま返す
            let mut result = Vec::new();
            for sensor in &self.sensors {
                if sensor.is_operational() {
                    result.extend(sensor.detected_targets.iter().cloned());
                }
            }
            result
        }
    }

    /// ネットワーク全体の検知統計
    /// 
    /// # 戻り値
    /// 
    /// 各センサーのIDとその検知統計のペアのベクター
    pub fn get_network_stats(&self) -> Vec<(String, DetectionStats)> {
        self.sensors
            .iter()
            .map(|sensor| (sensor.id.clone(), sensor.get_detection_stats()))
            .collect()
    }

    /// 特定のエリアをカバーしているセンサーを取得
    /// 
    /// 指定された中心位置と半径で定義されるエリアを
    /// 検知範囲でカバーしているセンサーを返します。
    /// 
    /// # 引数
    /// 
    /// * `center` - エリアの中心位置
    /// * `radius` - エリアの半径（メートル）
    /// 
    /// # 戻り値
    /// 
    /// 指定エリアをカバーするセンサーの参照ベクター
    pub fn get_sensors_covering_area(&self, center: Position3D, radius: f64) -> Vec<&Sensor> {
        self.sensors
            .iter()
            .filter(|sensor| {
                let distance = sensor.position.distance_3d(&center);
                distance <= sensor.detection_range + radius
            })
            .collect()
    }
}