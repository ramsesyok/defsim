use std::collections::HashMap;
use crate::models::{
    traits::{IAgent, IAllocator, IPlatform},
    common::{Position3D, AgentStatus},
    target::Target,
};

/// 優先度付けされたターゲット情報
#[derive(Debug, Clone)]
pub struct TargetPriority {
    pub target_id: String,
    pub tgo: f64,              // Time-to-go
    pub distance_xy: f64,      // XY距離
    pub assigned_missiles: u32, // 割り当て済みミサイル数
    pub target_endurance: u32,  // ターゲットの耐久値
}

/// 指揮所エージェント
#[derive(Debug)]
pub struct CommandPost {
    pub id: String,
    pub position: Position3D,
    pub arrival_radius: f64,      // 突破判定の到達範囲[m]
    pub status: AgentStatus,
    pub detected_targets: Vec<String>, // センサーから通知されたターゲットID
    pub missile_assignments: HashMap<String, Vec<String>>, // target_id -> [missile_id]
    pub target_priorities: Vec<TargetPriority>, // 優先度順のターゲットリスト
}

impl CommandPost {
    pub fn new(id: String, position: Position3D, arrival_radius: f64) -> Self {
        Self {
            id,
            position,
            arrival_radius,
            status: AgentStatus::Active,
            detected_targets: Vec::new(),
            missile_assignments: HashMap::new(),
            target_priorities: Vec::new(),
        }
    }

    /// センサーからのターゲット検知情報を受信
    pub fn receive_detections(&mut self, target_ids: Vec<String>) {
        self.detected_targets = target_ids;
    }

    /// ターゲットの優先度を計算（Tgo基準）
    pub fn calculate_target_priorities(&mut self, targets: &[Target]) {
        self.target_priorities.clear();

        for target in targets {
            if self.detected_targets.contains(&target.id) && target.is_active() {
                let tgo = self.calculate_tgo(target);
                let distance_xy = target.position.distance_xy(&self.position);
                let assigned_missiles = self.missile_assignments
                    .get(&target.id)
                    .map(|missiles| missiles.len() as u32)
                    .unwrap_or(0);

                let priority = TargetPriority {
                    target_id: target.id.clone(),
                    tgo,
                    distance_xy,
                    assigned_missiles,
                    target_endurance: target.endurance,
                };

                self.target_priorities.push(priority);
            }
        }

        // 優先度でソート: Tgo昇順 → XY距離昇順 → ID昇順
        self.target_priorities.sort_by(|a, b| {
            a.tgo.partial_cmp(&b.tgo)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.distance_xy.partial_cmp(&b.distance_xy).unwrap_or(std::cmp::Ordering::Equal))
                .then(a.target_id.cmp(&b.target_id))
        });
    }

    /// Tgo計算
    fn calculate_tgo(&self, target: &Target) -> f64 {
        target.calculate_time_to_go()
    }

    /// ランチャーを選定（クールダウン最短 → 距離最短 → ID昇順）
    pub fn select_best_launcher(
        &self, 
        launchers: &[Box<dyn IPlatform>], 
        target_position: Position3D
    ) -> Option<usize> {
        let mut best_launcher_index = None;
        let mut best_cooldown = f64::INFINITY;
        let mut best_distance = f64::INFINITY;
        let mut best_id = String::new();

        for (index, launcher) in launchers.iter().enumerate() {
            if launcher.can_launch() {
                let cooldown = launcher.get_cooldown_remaining();
                
                // ランチャーの位置を取得（仮実装：IPlatformに位置取得メソッドが必要）
                // ここでは簡略化のため、インデックスベースで距離を計算
                let distance = target_position.distance_xy(&self.position); // 仮の実装
                let launcher_id = format!("L{:03}", index + 1); // 仮のID生成

                let is_better = cooldown < best_cooldown ||
                    (cooldown == best_cooldown && distance < best_distance) ||
                    (cooldown == best_cooldown && distance == best_distance && launcher_id < best_id);

                if is_better {
                    best_launcher_index = Some(index);
                    best_cooldown = cooldown;
                    best_distance = distance;
                    best_id = launcher_id;
                }
            }
        }

        best_launcher_index
    }

    /// ミサイル割り当ての実行
    pub fn execute_assignments(&mut self, launchers: &mut [Box<dyn IPlatform>]) {
        for priority in &self.target_priorities {
            let assigned_count = priority.assigned_missiles;
            let target_endurance = priority.target_endurance;
            
            // 耐久度以上にミサイルを割り当てない
            if assigned_count >= target_endurance {
                continue;
            }

            // 追加で割り当てるミサイル数を決定
            let additional_missiles = (target_endurance - assigned_count).min(1); // 1発ずつ割り当て
            
            for _ in 0..additional_missiles {
                if let Some(launcher_index) = self.select_best_launcher(
                    launchers, 
                    Position3D::new(0.0, 0.0, 0.0) // 仮の位置（実際はターゲット位置）
                ) {
                    if let Some(missile) = launchers[launcher_index].launch(priority.target_id.clone()) {
                        // ミサイル割り当ての記録
                        let missile_id = missile.get_id();
                        self.missile_assignments
                            .entry(priority.target_id.clone())
                            .or_insert_with(Vec::new)
                            .push(missile_id);
                    }
                }
            }
        }
    }

    /// ミサイルが消滅した際の処理
    pub fn on_missile_destroyed(&mut self, missile_id: String) {
        for (_, missile_ids) in self.missile_assignments.iter_mut() {
            missile_ids.retain(|id| id != &missile_id);
        }
    }

    /// ターゲットが消滅した際の処理
    pub fn on_target_destroyed(&mut self, target_id: String) {
        self.missile_assignments.remove(&target_id);
        self.detected_targets.retain(|id| id != &target_id);
    }
}

impl IAgent for CommandPost {
    fn initialize(&mut self) {
        self.status = AgentStatus::Active;
    }

    fn tick(&mut self, _dt: f64) {
        // 指揮所は基本的に常にアクティブ
        // ターゲットの優先度計算とランチャーへの指示は
        // シミュレーションループから呼び出される
    }

    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn is_active(&self) -> bool {
        self.status == AgentStatus::Active
    }
}

impl IAllocator for CommandPost {
    fn allocate(&mut self, detected_targets: &[String], launchers: &mut [Box<dyn IPlatform>]) {
        // 検知されたターゲット情報を更新
        self.detected_targets = detected_targets.to_vec();
        
        // ここで実際のターゲット情報が必要だが、
        // シミュレーション全体の設計によって実装方法が変わる
        // とりあえずプレースホルダーとして基本的な処理を実装
        
        // ランチャーへの発射指示
        self.execute_assignments(launchers);
    }

    fn calculate_priority(&self, target_id: String) -> f64 {
        for priority in &self.target_priorities {
            if priority.target_id == target_id {
                return priority.tgo;
            }
        }
        f64::INFINITY
    }

    fn select_launcher(
        &self, 
        launchers: &[Box<dyn IPlatform>], 
        target_position: Position3D
    ) -> Option<usize> {
        self.select_best_launcher(launchers, target_position)
    }
}