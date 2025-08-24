use std::collections::HashMap;
use crate::models::{
    traits::{IAgent, IAllocator, IPlatform},
    common::{Position3D, AgentStatus},
    target::Target,
};

/// 優先度付けされたターゲット情報
/// 
/// ターゲットの脅威度を評価するための情報を格納します。
/// 優先度はTgo（Time-to-go）を基準とし、タイブレーカーとしてXY距離、ID順を使用します。
#[derive(Debug, Clone)]
pub struct TargetPriority {
    /// ターゲットの一意識別子
    pub target_id: String,
    /// Time-to-go: ターゲットが指揮所に到達するまでの予想時間（秒）
    pub tgo: f64,
    /// XY平面での指揮所からの距離（メートル）
    pub distance_xy: f64,
    /// このターゲットに既に割り当てられているミサイル数
    pub assigned_missiles: u32,
    /// ターゲットの耐久値（破壊に必要なミサイル数）
    pub target_endurance: u32,
}

/// 指揮所エージェント
/// 
/// 防御システムの中央統制を行うエージェントです。
/// センサーからのターゲット情報を基に脅威度を評価し、
/// ランチャーに対してミサイル発射指示を出します。
#[derive(Debug)]
pub struct CommandPost {
    /// 指揮所の一意識別子
    pub id: String,
    /// 指揮所の3次元位置
    pub position: Position3D,
    /// ターゲットの突破判定に使用される到達範囲（メートル）
    pub arrival_radius: f64,
    /// 指揮所の現在のステータス
    pub status: AgentStatus,
    /// センサーから通知されたターゲットIDのリスト
    pub detected_targets: Vec<String>,
    /// ターゲットIDからミサイルIDのリストへのマッピング
    pub missile_assignments: HashMap<String, Vec<String>>,
    /// 優先度順に並べられたターゲットのリスト
    pub target_priorities: Vec<TargetPriority>,
}

impl CommandPost {
    /// 新しい指揮所を作成します
    /// 
    /// # 引数
    /// 
    /// * `id` - 指揮所の一意識別子
    /// * `position` - 指揮所の3次元位置
    /// * `arrival_radius` - ターゲットの突破判定範囲（メートル）
    /// 
    /// # 戻り値
    /// 
    /// 初期化された指揮所インスタンス
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
    /// 
    /// # 引数
    /// 
    /// * `target_ids` - 検知されたターゲットIDのリスト
    pub fn receive_detections(&mut self, target_ids: Vec<String>) {
        self.detected_targets = target_ids;
    }

    /// ターゲットの優先度を計算（Tgo基準）
    /// 
    /// 検知されたアクティブなターゲットに対して脅威度を計算し、
    /// Tgo（Time-to-go）の昇順、XY距離の昇順、ID昇順でソートします。
    /// 
    /// # 引数
    /// 
    /// * `targets` - 評価対象のターゲットのスライス
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

    /// Tgo（Time-to-go）を計算
    /// 
    /// ターゲットが指揮所に到達するまでの予想時間を算出します。
    /// 
    /// # 引数
    /// 
    /// * `target` - 計算対象のターゲット
    /// 
    /// # 戻り値
    /// 
    /// 到達予想時間（秒）
    fn calculate_tgo(&self, target: &Target) -> f64 {
        target.calculate_time_to_go()
    }

    /// ランチャーを選定（クールダウン最短 → 距離最短 → ID昇順）
    /// 
    /// 発射可能なランチャーの中から、クールダウン最短、
    /// 距離最短、ID昇順の優先度で最適ランチャーを選定します。
    /// 
    /// # 引数
    /// 
    /// * `launchers` - 選定対象ランチャーのスライス
    /// * `target_position` - ターゲットの位置
    /// 
    /// # 戻り値
    /// 
    /// 選定されたランチャーのインデックス、発射可能なランチャーがない場合はNone
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
    /// 
    /// 優先度順のターゲットに対して、耐久度を超えない範囲で
    /// ミサイルを順次割り当てて発射します。
    /// 
    /// # 引数
    /// 
    /// * `launchers` - ミサイル発射を行うランチャーの可変スライス
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


    /// ターゲットリストの更新
    /// 
    /// アクティブなターゲットの情報をもとに優先度リストを再構築します。
    /// 
    /// # 引数
    /// 
    /// * `targets` - 更新対象のターゲットの参照ベクター
    pub fn update_target_list(&mut self, targets: Vec<&Target>) {
        self.target_priorities.clear();
        
        for target in targets {
            if target.is_active() {
                let tgo = target.calculate_time_to_go();
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

        self.target_priorities.sort_by(|a, b| {
            a.tgo.partial_cmp(&b.tgo)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.distance_xy.partial_cmp(&b.distance_xy).unwrap_or(std::cmp::Ordering::Equal))
                .then(a.target_id.cmp(&b.target_id))
        });
    }

    /// ミサイル発射割り当てを取得
    /// 
    /// 指定されたランチャーに対して、発射すべきミサイルの割り当て情報を返します。
    /// 
    /// # 引数
    /// 
    /// * `launcher_id` - ランチャーのID
    /// 
    /// # 戻り値
    /// 
    /// ミサイル割り当て情報、割り当て可能なターゲットがない場合はNone
    pub fn get_missile_assignment(&mut self, launcher_id: &str) -> Option<crate::simulation::MissileAssignment> {
        for priority in &self.target_priorities {
            let assigned_count = priority.assigned_missiles;
            if assigned_count < priority.target_endurance {
                return Some(crate::simulation::MissileAssignment {
                    launcher_id: launcher_id.to_string(),
                    target_id: priority.target_id.clone(),
                    priority: priority.tgo,
                });
            }
        }
        None
    }

    /// ミサイルが消滅した際の処理
    /// 
    /// 指定されたミサイルIDを割り当てリストから除去します。
    /// これにより割り当て数が正しく管理されます。
    /// 
    /// # 引数
    /// 
    /// * `missile_id` - 消滅したミサイルのID
    pub fn on_missile_destroyed(&mut self, missile_id: String) {
        for (_, missile_ids) in self.missile_assignments.iter_mut() {
            missile_ids.retain(|id| id != &missile_id);
        }
    }

    /// ターゲットが消滅した際の処理
    /// 
    /// 指定されたターゲットに関連するすべての情報をクリアします。
    /// ミサイル割り当てと検知リストから除去されます。
    /// 
    /// # 引数
    /// 
    /// * `target_id` - 消滅したターゲットのID
    pub fn on_target_destroyed(&mut self, target_id: String) {
        self.missile_assignments.remove(&target_id);
        self.detected_targets.retain(|id| id != &target_id);
    }
}

impl IAgent for CommandPost {
    fn initialize(&mut self, scenario_config: &crate::scenario::ScenarioConfig) {
        self.status = AgentStatus::Active;
        
        // ポリシー設定の適用
        let policy = &scenario_config.policy;
        if !policy.tgo_definition.is_empty() {
            // Tgoの定義に基づく計算方法を設定
        }
        // tie_breakers、launcher_selection_order、launcher_initially_cooledの設定も必要に応じて実装
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