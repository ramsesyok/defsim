use std::collections::HashSet;
use crate::models::{
    traits::{IAgent, ISensor},
    common::{Position3D, AgentStatus},
    target::Target,
};

/// センサーエージェント
#[derive(Debug, Clone)]
pub struct Sensor {
    pub id: String,
    pub position: Position3D,
    pub detection_range: f64,    // 探知範囲[m] (球形半径)
    pub status: AgentStatus,
    pub detected_targets: HashSet<String>, // 現在検知中のターゲットID
    pub detection_history: Vec<DetectionEvent>, // 検知履歴
}

/// 検知イベント
#[derive(Debug, Clone)]
pub struct DetectionEvent {
    pub timestamp: f64,
    pub target_id: String,
    pub target_position: Position3D,
    pub distance: f64,
    pub event_type: DetectionEventType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DetectionEventType {
    FirstDetected,  // 初回検知
    Tracking,       // 追跡中
    Lost,          // ロスト
}

impl Sensor {
    pub fn new(
        id: String,
        position: Position3D,
        detection_range: f64,
    ) -> Self {
        Self {
            id,
            position,
            detection_range,
            status: AgentStatus::Active,
            detected_targets: HashSet::new(),
            detection_history: Vec::new(),
        }
    }

    /// ターゲットの検知処理
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
    pub fn distance_to_target(&self, target: &Target) -> f64 {
        self.position.distance_3d(&target.position)
    }

    /// センサーの有効性チェック
    pub fn is_operational(&self) -> bool {
        self.status == AgentStatus::Active
    }

    /// 検知履歴の取得（最新N件）
    pub fn get_recent_detections(&self, count: usize) -> Vec<&DetectionEvent> {
        let start_index = if self.detection_history.len() > count {
            self.detection_history.len() - count
        } else {
            0
        };
        
        self.detection_history[start_index..].iter().collect()
    }

    /// 検知統計の計算
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
    pub fn clear_detection_history(&mut self) {
        self.detection_history.clear();
    }

    /// 検知範囲内かどうかの判定
    pub fn is_in_detection_range(&self, position: Position3D) -> bool {
        self.position.distance_3d(&position) <= self.detection_range
    }
}

/// 検知統計情報
#[derive(Debug, Clone)]
pub struct DetectionStats {
    pub total_detections: usize,
    pub first_detections: usize,
    pub lost_detections: usize,
    pub currently_tracking: usize,
}

impl IAgent for Sensor {
    fn initialize(&mut self) {
        self.status = AgentStatus::Active;
        self.detected_targets.clear();
        self.detection_history.clear();
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
#[derive(Debug)]
pub struct SensorNetwork {
    pub sensors: Vec<Sensor>,
    pub fusion_enabled: bool, // データ融合機能
}

impl SensorNetwork {
    pub fn new() -> Self {
        Self {
            sensors: Vec::new(),
            fusion_enabled: true,
        }
    }

    pub fn add_sensor(&mut self, sensor: Sensor) {
        self.sensors.push(sensor);
    }

    /// ネットワーク全体でのターゲット検知
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
    pub fn get_network_stats(&self) -> Vec<(String, DetectionStats)> {
        self.sensors
            .iter()
            .map(|sensor| (sensor.id.clone(), sensor.get_detection_stats()))
            .collect()
    }

    /// 特定のエリアをカバーしているセンサーを取得
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