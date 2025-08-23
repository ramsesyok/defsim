use std::collections::VecDeque;
use crate::models::{
    traits::{IAgent, IPlatform},
    common::{Position3D, AgentStatus},
    missile::Missile,
};

/// 発射記録
#[derive(Debug, Clone)]
pub struct LaunchRecord {
    pub timestamp: f64,
    pub missile_id: String,
    pub target_id: String,
    pub launch_position: Position3D,
}

/// ランチャーエージェント
#[derive(Debug)]
pub struct Launcher {
    pub id: String,
    pub position: Position3D,
    pub status: AgentStatus,
    pub max_missiles: u32,           // 最大装備ミサイル数
    pub current_missiles: u32,       // 現在の装備ミサイル数
    pub cooldown_time: f64,          // クールダウン時間[s]
    pub cooldown_remaining: f64,     // 残りクールダウン時間[s]
    pub launch_queue: VecDeque<String>, // 発射待ちターゲットID
    pub launch_history: Vec<LaunchRecord>, // 発射履歴
    pub missile_counter: u32,        // ミサイルID生成用カウンタ
    
    // ミサイル性能パラメータ（このランチャーが発射するミサイルの仕様）
    pub missile_initial_speed: f64,   // 初速[m/s]
    pub missile_max_speed: f64,       // 最大速度[m/s]
    pub missile_max_accel: f64,       // 最大加速度[m/s²]
    pub missile_max_turn_rate: f64,   // 最大旋回レート[deg/s]
    pub missile_intercept_radius: f64, // 迎撃判定距離[m]
}

impl Launcher {
    pub fn new(id: String, position: Position3D) -> Self {
        Self {
            id,
            position,
            status: AgentStatus::Active,
            max_missiles: 0,                    // initializeで設定
            current_missiles: 0,                // initializeで設定
            cooldown_time: 0.0,                 // initializeで設定
            cooldown_remaining: 0.0,            // initializeで設定
            launch_queue: VecDeque::new(),
            launch_history: Vec::new(),
            missile_counter: 0,
            missile_initial_speed: 0.0,         // initializeで設定
            missile_max_speed: 0.0,             // initializeで設定
            missile_max_accel: 0.0,             // initializeで設定
            missile_max_turn_rate: 0.0,         // initializeで設定
            missile_intercept_radius: 0.0,      // initializeで設定
        }
    }

    /// ミサイル発射の実行
    pub fn fire_missile(&mut self, target_id: String, current_time: f64) -> Option<Missile> {
        if !self.can_launch() {
            return None;
        }

        // ミサイルID生成
        self.missile_counter += 1;
        let missile_id = format!("{}_M{:03}", self.id, self.missile_counter);

        // ミサイル作成
        let missile = Missile::new(
            missile_id.clone(),
            self.position,
            target_id.clone(),
        );

        // ランチャー状態更新
        self.current_missiles -= 1;
        self.cooldown_remaining = self.cooldown_time;

        // 発射記録
        let launch_record = LaunchRecord {
            timestamp: current_time,
            missile_id: missile_id.clone(),
            target_id: target_id.clone(),
            launch_position: self.position,
        };
        self.launch_history.push(launch_record);

        Some(missile)
    }

    /// 発射待ちキューにターゲットを追加
    pub fn queue_target(&mut self, target_id: String) {
        if !self.launch_queue.contains(&target_id) {
            self.launch_queue.push_back(target_id);
        }
    }

    /// 発射待ちキューから次のターゲットを取得
    pub fn get_next_target(&mut self) -> Option<String> {
        self.launch_queue.pop_front()
    }

    /// 発射待ちキューをクリア
    pub fn clear_queue(&mut self) {
        self.launch_queue.clear();
    }

    /// 特定のターゲットをキューから削除
    pub fn remove_target_from_queue(&mut self, target_id: &str) {
        self.launch_queue.retain(|id| id != target_id);
    }

    /// ミサイル再装填（補給処理）
    pub fn reload(&mut self, count: u32) {
        let reload_count = count.min(self.max_missiles - self.current_missiles);
        self.current_missiles += reload_count;
    }

    /// 満載まで再装填
    pub fn reload_full(&mut self) {
        self.current_missiles = self.max_missiles;
    }


    /// ミサイル発射（シミュレーションエンジン用）
    pub fn fire_missile_at_target(&mut self, target_id: &str) -> Option<Missile> {
        // 直接Missileを作成して返す
        if !self.can_launch() {
            return None;
        }

        // ミサイルを発射
        self.current_missiles -= 1;
        self.cooldown_remaining = self.cooldown_time;
        self.missile_counter += 1;

        let missile_id = format!("{}_{:03}", self.id, self.missile_counter);
        let missile = Missile::new(
            missile_id,
            self.position,
            target_id.to_string(),
        );

        // 発射記録を追加
        let launch_record = LaunchRecord {
            timestamp: 0.0, // 実際の時刻は外部から設定
            missile_id: missile.get_id(),
            target_id: target_id.to_string(),
            launch_position: self.position,
        };
        self.launch_history.push(launch_record);

        Some(missile)
    }

    /// 発射統計の取得
    pub fn get_launch_stats(&self) -> LaunchStats {
        let total_launches = self.launch_history.len();
        let missiles_remaining = self.current_missiles as usize;
        let missiles_fired = (self.max_missiles - self.current_missiles) as usize;
        let queue_length = self.launch_queue.len();

        LaunchStats {
            total_launches,
            missiles_remaining,
            missiles_fired,
            queue_length,
            cooldown_remaining: self.cooldown_remaining,
            is_ready: self.can_launch(),
        }
    }

    /// 最近の発射記録を取得
    pub fn get_recent_launches(&self, count: usize) -> Vec<&LaunchRecord> {
        let start_index = if self.launch_history.len() > count {
            self.launch_history.len() - count
        } else {
            0
        };
        
        self.launch_history[start_index..].iter().collect()
    }

    /// ランチャーの効率性を計算
    pub fn calculate_efficiency(&self, time_elapsed: f64) -> f64 {
        if time_elapsed <= 0.0 {
            return 0.0;
        }
        
        let theoretical_max_launches = (time_elapsed / self.cooldown_time).floor() as usize;
        let actual_launches = self.launch_history.len();
        
        if theoretical_max_launches > 0 {
            (actual_launches as f64) / (theoretical_max_launches as f64)
        } else {
            0.0
        }
    }

    /// ランチャーの位置を取得（IPlatform実装で必要）
    pub fn get_position(&self) -> Position3D {
        self.position
    }

    /// ターゲットまでの距離を計算
    pub fn distance_to_target(&self, target_position: Position3D) -> f64 {
        self.position.distance_xy(&target_position)
    }
}

/// 発射統計情報
#[derive(Debug, Clone)]
pub struct LaunchStats {
    pub total_launches: usize,
    pub missiles_remaining: usize,
    pub missiles_fired: usize,
    pub queue_length: usize,
    pub cooldown_remaining: f64,
    pub is_ready: bool,
}

impl IAgent for Launcher {
    fn initialize(&mut self, scenario_config: &crate::scenario::ScenarioConfig) {
        self.status = AgentStatus::Active;
        self.launch_queue.clear();
        self.launch_history.clear();
        
        // シナリオからランチャー設定を探して適用
        for launcher_config in &scenario_config.friendly_forces.launchers {
            if launcher_config.id == self.id {
                // ミサイル装備数の設定
                self.max_missiles = launcher_config.missiles_loaded;
                self.current_missiles = launcher_config.missiles_loaded;
                
                // クールダウン時間の設定
                self.cooldown_time = launcher_config.cooldown_s;
                
                // 初期クールダウン状態の設定
                if scenario_config.policy.launcher_initially_cooled {
                    self.cooldown_remaining = 0.0;
                } else {
                    self.cooldown_remaining = launcher_config.cooldown_s;
                }
                break;
            }
        }
        
        // ミサイル性能パラメータの設定
        let missile_kinematics = &scenario_config.missile_defaults.kinematics;
        self.missile_initial_speed = missile_kinematics.initial_speed_mps;
        self.missile_max_speed = missile_kinematics.max_speed_mps;
        self.missile_max_accel = missile_kinematics.max_accel_mps2;
        self.missile_max_turn_rate = missile_kinematics.max_turn_rate_deg_s;
        self.missile_intercept_radius = missile_kinematics.intercept_radius_m;
    }

    fn tick(&mut self, dt: f64) {
        if self.status != AgentStatus::Active {
            return;
        }

        // クールダウンタイマーの更新
        if self.cooldown_remaining > 0.0 {
            self.cooldown_remaining = (self.cooldown_remaining - dt).max(0.0);
        }

        // 自動発射処理（キューがある場合）
        if self.can_launch() && !self.launch_queue.is_empty() {
            if let Some(target_id) = self.get_next_target() {
                // 実際の発射処理は外部（シミュレーションループ）から呼ばれることを想定
                // ここでは発射可能状態の維持のみ行う
                self.queue_target(target_id); // キューに戻す（外部で処理されるまで）
            }
        }
    }

    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn is_active(&self) -> bool {
        self.status == AgentStatus::Active
    }
}

impl IPlatform for Launcher {
    fn launch(&mut self, target_id: String) -> Option<Box<dyn IAgent>> {
        if !self.can_launch() {
            return None;
        }

        // 現在時刻の取得（簡略実装）
        static mut CURRENT_TIME: f64 = 0.0;
        let current_time = unsafe { CURRENT_TIME };

        if let Some(missile) = self.fire_missile(target_id, current_time) {
            // Box<dyn IAgent>として返すため、型変換
            Some(Box::new(missile) as Box<dyn IAgent>)
        } else {
            None
        }
    }

    fn assign(&mut self, target_id: String) -> bool {
        if self.current_missiles > 0 {
            self.queue_target(target_id);
            true
        } else {
            false
        }
    }

    fn can_launch(&self) -> bool {
        self.status == AgentStatus::Active &&
        self.current_missiles > 0 &&
        self.cooldown_remaining <= 0.0
    }

    fn get_remaining_missiles(&self) -> u32 {
        self.current_missiles
    }

    fn get_cooldown_remaining(&self) -> f64 {
        self.cooldown_remaining
    }
}

/// 複数のランチャーを管理するバッテリー
#[derive(Debug)]
pub struct LauncherBattery {
    pub id: String,
    pub launchers: Vec<Launcher>,
    pub battery_position: Position3D,
}

impl LauncherBattery {
    pub fn new(id: String, battery_position: Position3D) -> Self {
        Self {
            id,
            launchers: Vec::new(),
            battery_position,
        }
    }

    pub fn add_launcher(&mut self, launcher: Launcher) {
        self.launchers.push(launcher);
    }

    /// バッテリー全体の発射可能ミサイル数
    pub fn total_available_missiles(&self) -> u32 {
        self.launchers.iter().map(|l| l.current_missiles).sum()
    }

    /// 発射可能なランチャー数
    pub fn ready_launchers_count(&self) -> usize {
        self.launchers.iter().filter(|l| l.can_launch()).count()
    }

    /// 最適なランチャーを選択（クールダウン最短 → 距離最短 → ID昇順）
    pub fn select_best_launcher(&self, target_position: Position3D) -> Option<usize> {
        let mut best_index = None;
        let mut best_cooldown = f64::INFINITY;
        let mut best_distance = f64::INFINITY;
        let mut best_id = String::new();

        for (index, launcher) in self.launchers.iter().enumerate() {
            if launcher.can_launch() {
                let cooldown = launcher.cooldown_remaining;
                let distance = launcher.distance_to_target(target_position);
                let launcher_id = &launcher.id;

                let is_better = cooldown < best_cooldown ||
                    (cooldown == best_cooldown && distance < best_distance) ||
                    (cooldown == best_cooldown && distance == best_distance && launcher_id < &best_id);

                if is_better {
                    best_index = Some(index);
                    best_cooldown = cooldown;
                    best_distance = distance;
                    best_id = launcher_id.clone();
                }
            }
        }

        best_index
    }

    /// バッテリー全体の統計
    pub fn get_battery_stats(&self) -> BatteryStats {
        let total_launchers = self.launchers.len();
        let active_launchers = self.launchers.iter().filter(|l| l.is_active()).count();
        let ready_launchers = self.ready_launchers_count();
        let total_missiles = self.total_available_missiles();
        let total_launches = self.launchers.iter().map(|l| l.launch_history.len()).sum();

        BatteryStats {
            total_launchers,
            active_launchers,
            ready_launchers,
            total_missiles,
            total_launches,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BatteryStats {
    pub total_launchers: usize,
    pub active_launchers: usize,
    pub ready_launchers: usize,
    pub total_missiles: u32,
    pub total_launches: usize,
}