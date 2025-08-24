//! # Simulation モジュール
//! 
//! 防衛シミュレーションの中核となるシミュレーションエンジンを提供します。
//! 
//! このモジュールは、時間駆動シミュレーションのメインループを管理し、
//! すべてのエージェント（ターゲット、センサー、指揮所、ランチャー、ミサイル）の
//! 協調動作を制御します。固定時間刻み（Δt）による数値積分でリアルタイム性を
//! 保ちながら、複雑な防衛戦術の実行過程を再現します。
//! 
//! ## 主要機能
//! 
//! - **シミュレーションループ管理**: 固定時間刻みによる時間進行制御
//! - **エージェント統合管理**: 全エージェントのライフサイクル管理
//! - **戦術処理順序制御**: 最適な処理順序でのエージェント更新
//! - **パフォーマンス監視**: 実行進行状況と統計情報の提供
//! 
//! ## シミュレーション処理順序
//! 
//! 各時間刻みにおいて、以下の順序で処理が実行されます：
//! 
//! 1. **ターゲット処理**: 敵機の移動、到達判定、領域外判定
//! 2. **ミサイル処理**: 誘導計算、運動更新、衝突判定
//! 3. **センサー処理**: ターゲット検知、検知状態更新
//! 4. **指揮所処理**: 優先度評価、ミサイル割り当て決定
//! 5. **ランチャー処理**: ミサイル発射、クールダウン管理
//! 
//! この順序により、戦術的に整合性の取れた防衛行動が再現されます。
//! 
//! ## 使用例
//! 
//! ```rust
//! use defsim::simulation::SimulationEngine;
//! use defsim::scenario::ScenarioConfig;
//! 
//! // シナリオファイルを読み込み
//! let config = ScenarioConfig::from_file("scenarios/basic_defense.yaml")?;
//! 
//! // シミュレーションエンジンを作成
//! let mut engine = SimulationEngine::new(config, 1); // verbose_level=1
//! 
//! // 初期化とシミュレーション実行
//! engine.initialize()?;
//! engine.run()?;
//! ```

use crate::models::{Position3D as ModelPosition3D, *};
use crate::scenario::*;
use tracing::{info, warn, error, debug, trace};

pub struct SimulationEngine {
    pub current_time: f64,
    pub dt: f64,
    pub max_time: f64,
    pub seed: u64,
    pub step_count: u64,
    
    pub command_post: CommandPost,
    pub sensors: Vec<Sensor>,
    pub launchers: Vec<Launcher>,
    pub targets: Vec<Target>,
    pub missiles: Vec<Missile>,
    
    pub scenario_config: ScenarioConfig,
    pub verbose_level: u8,
}

impl SimulationEngine {
    pub fn new(scenario: ScenarioConfig, verbose_level: u8) -> Self {
        let dt = scenario.sim.dt_s;
        let max_time = scenario.sim.t_max_s;
        let seed = scenario.sim.seed;
        
        let command_post_pos = ModelPosition3D::new(
            scenario.command_post.position.x_m,
            scenario.command_post.position.y_m,
            0.0
        );
        
        let command_post = CommandPost::new(
            "CP001".to_string(),
            command_post_pos,
            scenario.command_post.arrival_radius_m,
        );
        
        Self {
            current_time: 0.0,
            dt,
            max_time,
            seed,
            step_count: 0,
            command_post,
            sensors: Vec::new(),
            launchers: Vec::new(),
            targets: Vec::new(),
            missiles: Vec::new(),
            scenario_config: scenario,
            verbose_level,
        }
    }
    
    pub fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.verbose_level > 0 {
            info!("シミュレーションエンジンを初期化中...");
        }
        
        self.initialize_command_post()?;
        self.initialize_sensors()?;
        self.initialize_launchers()?;
        self.initialize_enemy_groups()?;
        
        if self.verbose_level > 0 {
            info!("初期化完了:");
            info!("  指揮所: 1基");
            info!("  センサー: {}基", self.sensors.len());
            info!("  ランチャー: {}基", self.launchers.len());
            info!("  敵機: {}機", self.targets.len());
        }
        
        Ok(())
    }
    
    fn initialize_command_post(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.command_post.initialize(&self.scenario_config);
        
        if self.verbose_level > 1 {
            debug!("指揮所初期化: {} (位置: {:.0}, {:.0})", 
                    self.command_post.get_id(),
                    self.command_post.position.x,
                    self.command_post.position.y);
        }
        
        Ok(())
    }
    
    fn initialize_sensors(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for sensor_config in &self.scenario_config.friendly_forces.sensors {
            let sensor_pos = ModelPosition3D::new(
                sensor_config.pos.x_m,
                sensor_config.pos.y_m,
                sensor_config.pos.z_m,
            );
            
            let mut sensor = Sensor::new(
                sensor_config.id.clone(),
                sensor_pos,
            );
            
            sensor.initialize(&self.scenario_config);
            
            if self.verbose_level > 1 {
                debug!("センサー初期化: {} (範囲: {:.0}m)", 
                        sensor.get_id(), 
                        sensor_config.range_m);
            }
            
            self.sensors.push(sensor);
        }
        
        Ok(())
    }
    
    fn initialize_launchers(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for launcher_config in &self.scenario_config.friendly_forces.launchers {
            let launcher_pos = ModelPosition3D::new(
                launcher_config.pos.x_m,
                launcher_config.pos.y_m,
                launcher_config.pos.z_m,
            );
            
            let mut launcher = Launcher::new(
                launcher_config.id.clone(),
                launcher_pos,
            );
            
            launcher.initialize(&self.scenario_config);
            
            if self.verbose_level > 1 {
                debug!("ランチャー初期化: {} (ミサイル: {}発)", 
                        launcher.get_id(), 
                        launcher_config.missiles_loaded);
            }
            
            self.launchers.push(launcher);
        }
        
        Ok(())
    }
    
    fn initialize_enemy_groups(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // シナリオ設定に定義された全ての敵グループ設定をループ処理
        // 各グループは独立した配置パターン、出現時刻、パラメータを持つ
        for group_config in &self.scenario_config.enemy_forces.groups {
            // === グループの配置中心位置を設定 ===
            // シナリオ設定のXY座標とZ座標から3次元位置を構築
            let group_center = ModelPosition3D::new(
                group_config.center_xy.x_m,
                group_config.center_xy.y_m,
                group_config.z_m,
            );
            
            // === TargetGroupオブジェクトを構築 ===
            // グループ全体の配置パターン（同心円リング配置）と動作パラメータを設定
            let target_group = TargetGroup {
                id: group_config.id.clone(),                    // グループ識別子
                center_position: group_center,                  // 配置中心位置
                count: group_config.count,                      // グループ内ターゲット数
                ring_spacing: group_config.ring_spacing_m,     // リング間隔（メートル）
                start_angle: group_config.start_angle_deg,     // 配置開始角度（度）
                ring_half_offset: group_config.ring_half_offset, // 外側リングの半角オフセット
                endurance: group_config.endurance_pt,          // 各ターゲットの耐久値
                spawn_time: group_config.spawn_time_s,         // グループ出現時刻（秒）
                speed: group_config.speed_mps,                 // 移動速度（m/s）
                // 全ターゲットの共通目的地（指揮所位置）
                destination: ModelPosition3D::new(
                    self.scenario_config.command_post.position.x_m,
                    self.scenario_config.command_post.position.y_m,
                    0.0  // 指揮所は地上レベル
                ),
                arrival_radius: self.scenario_config.command_post.arrival_radius_m,
            };
            
            // === グループ内の個別ターゲット生成 ===
            // TargetGroupの配置アルゴリズムに基づいて個別のTargetインスタンスを生成
            // 同心円リング配置：中心1個 + 外側リング上に等角度間隔配置
            let targets = target_group.generate_targets();
            
            // === 生成された各ターゲットの初期化とシミュレーションへの登録 ===
            // 各ターゲットは個別のID（グループID_T001形式）を持つ
            for mut target in targets {
                // シナリオ設定に基づいて各ターゲットの詳細パラメータを設定
                target.initialize(&self.scenario_config);
                // シミュレーションエンジンのターゲットリストに追加
                self.targets.push(target);
            }
            
            // デバッグレベルのログ出力（グループ単位の初期化完了通知）
            if self.verbose_level > 1 {
                debug!("敵グループ初期化: {} ({}機, 出現時刻: {:.1}秒)", 
                        group_config.id, 
                        group_config.count, 
                        group_config.spawn_time_s);
            }
        }
        
        Ok(())
    }
    
    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("=== シミュレーション実行開始 ===");
        
        while self.current_time < self.max_time {
            self.step();
            
            if self.verbose_level > 2 {
                trace!("時刻: {:.1}秒 (ステップ: {})", self.current_time, self.step_count);
            }
            
            if self.step_count % 100 == 0 && self.verbose_level > 0 {
                let progress = (self.current_time / self.max_time) * 100.0;
                info!("進行状況: {:.1}% ({:.1}/{:.1}秒)", progress, self.current_time, self.max_time);
            }
            
            if self.step_count > 10000 {
                break;
            }
        }
        
        info!("=== シミュレーション完了 ===");
        info!("実行時間: {:.1}秒", self.current_time);
        info!("総ステップ数: {}", self.step_count);
        
        Ok(())
    }
    
    fn step(&mut self) {
        self.process_targets();
        self.process_missiles();
        self.process_sensors();
        self.process_command_post();
        self.process_launchers();
        
        self.current_time += self.dt;
        self.step_count += 1;
    }
    
    fn process_targets(&mut self) {
        for target in &mut self.targets {
            if target.is_active() && self.current_time >= target.spawn_time {
                target.tick(self.dt);
            }
        }
    }
    
    fn process_missiles(&mut self) {
        for missile in &mut self.missiles {
            if missile.is_active() {
                missile.tick(self.dt);
            }
        }
        
        self.missiles.retain(|m| m.is_active());
    }
    
    fn process_sensors(&mut self) {
        for sensor in &mut self.sensors {
            if sensor.is_active() {
                sensor.update_detections(&self.targets, self.current_time);
                sensor.tick(self.dt);
            }
        }
    }
    
    fn process_command_post(&mut self) {
        if self.command_post.is_active() {
            let detected_targets: Vec<String> = self.sensors
                .iter()
                .flat_map(|s| s.get_detected_targets())
                .collect();
            
            let active_targets: Vec<&Target> = self.targets
                .iter()
                .filter(|t| t.is_active() && detected_targets.contains(&t.get_id()))
                .collect();
            
            self.command_post.update_target_list(active_targets);
            self.command_post.tick(self.dt);
        }
    }
    
    fn process_launchers(&mut self) {
        for launcher in &mut self.launchers {
            if launcher.is_active() {
                if let Some(assignment) = self.command_post.get_missile_assignment(&launcher.get_id()) {
                    if let Some(mut new_missile) = launcher.fire_missile_at_target(&assignment.target_id) {
                        new_missile.initialize(&self.scenario_config);
                        self.missiles.push(new_missile);
                    }
                }
                launcher.tick(self.dt);
            }
        }
    }
}

pub struct MissileAssignment {
    pub launcher_id: String,
    pub target_id: String,
    pub priority: f64,
}