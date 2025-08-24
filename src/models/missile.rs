use crate::models::{
    traits::{IAgent, IMovable, IMissile, ICollision},
    common::{Position3D, Velocity3D, Acceleration3D, AgentStatus, math_utils},
};
use tracing::{info, warn, error, debug, trace};

/// ミサイル誘導フェーズ
/// 
/// ミサイルの飛翼段階を表し、各段階で異なる誘導アルゴリズムや行動が適用されます。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GuidancePhase {
    /// ブースト段階（初期加速フェーズ）
    Boost,
    /// 中間段階（巡航フェーズ）
    Midcourse,
    /// 終盤段階（ターゲット近接フェーズ）
    Endgame,
}

/// ミサイル終了理由
/// 
/// ミサイルがアクティブ状態から終了した理由を表します。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MissileEndReason {
    /// ターゲットに命中した
    Hit,
    /// 自爆した（miss distanceが継続的に増加したため）
    SelfDestruct,
    /// ターゲットが消失した
    TargetLost,
    /// シミュレーション領域外に出た
    OutOfBounds,
}

/// ミサイルエージェント
/// 
/// ランチャーから発射され、ターゲットに向かって誘導されるミサイルです。
/// True 3D比例航法(N=3-4)を使用し、終盤でのmiss distance増加時に自爆します。
#[derive(Debug, Clone)]
pub struct Missile {
    pub id: String,
    pub position: Position3D,
    pub velocity: Velocity3D,
    pub acceleration: Acceleration3D,
    pub target_id: String,
    pub status: AgentStatus,
    
    /// 初速（m/s）
    pub initial_speed: f64,
    /// 最大速度（m/s）
    pub max_speed: f64,
    /// 最大加速度（m/s²）
    pub max_accel: f64,
    /// 最大旋回レート（deg/s）
    pub max_turn_rate: f64,
    /// 迎撃判定距離（m）
    pub intercept_radius: f64,
    
    /// 比例航法定数（通常3-4）
    pub guidance_n: f64,
    /// 現在の誘導フェーズ
    pub guidance_phase: GuidancePhase,
    /// 終盤判定距離閾値（intercept_radius × 倍数）
    pub endgame_threshold: f64,
    
    /// miss distanceの履歴（自爆判定用）
    pub miss_distance_history: Vec<f64>,
    /// miss distance増加の連続回数
    pub miss_increase_count: u32,
    /// 終盤でのmiss distance増加判定ティック数
    pub endgame_miss_increase_ticks: u32,
    
    /// ミサイルの姿勢
    pub attitude: Attitude3D,
    /// 現在の旋回レート（deg/s）
    pub turn_rate: f64,
    
    /// 飛翔時間（秒）
    pub flight_time: f64,
    /// 累積飛行距離（m）
    pub total_distance: f64,
    /// 終了理由
    pub end_reason: Option<MissileEndReason>,
}

/// 3次元姿勢
/// 
/// ミサイルの空間内での姿勢をオイラー角で表現します。
#[derive(Debug, Clone, Copy)]
pub struct Attitude3D {
    /// ピッチ角（上下方向の傾き、度）
    pub pitch: f64,
    /// ヨー角（左右方向の傾き、度）
    pub yaw: f64,
    /// ロール角（回転角、度）
    pub roll: f64,
}

impl Attitude3D {
    /// 新しい3次元姿勢を作成
    /// 
    /// # 引数
    /// 
    /// * `pitch` - ピッチ角（度）
    /// * `yaw` - ヨー角（度）
    /// * `roll` - ロール角（度）
    /// 
    /// # 戻り値
    /// 
    /// 新しいAttitude3Dインスタンス
    pub fn new(pitch: f64, yaw: f64, roll: f64) -> Self {
        Self { pitch, yaw, roll }
    }
    
    /// 速度ベクトルから姿勢を計算
    /// 
    /// 速度ベクトルの方向からピッチ角とヨー角を計算します。
    /// ロール角は簡略化のため0に設定されます。
    /// 
    /// # 引数
    /// 
    /// * `velocity` - 姿勢計算の基準となる速度ベクトル
    /// 
    /// # 戻り値
    /// 
    /// 計算された姿勢
    pub fn from_velocity(velocity: &Velocity3D) -> Self {
        let speed_xy = velocity.magnitude_xy();
        let pitch = if speed_xy > 0.0 {
            math_utils::rad_to_deg(velocity.z.atan2(speed_xy))
        } else {
            0.0
        };
        
        let yaw = if velocity.x.abs() > 1e-10 || velocity.y.abs() > 1e-10 {
            math_utils::rad_to_deg(velocity.y.atan2(velocity.x))
        } else {
            0.0
        };
        
        Self::new(pitch, yaw, 0.0) // ロールは簡略化のため0
    }
}

impl Missile {
    /// 新しいミサイルを作成します
    /// 
    /// # 引数
    /// 
    /// * `id` - ミサイルの一意識別子
    /// * `launch_position` - 発射位置
    /// * `target_id` - ターゲットのID
    /// 
    /// # 戻り値
    /// 
    /// 初期化されたミサイルインスタンス（initializeメソッドで詳細設定が必要）
    pub fn new(
        id: String,
        launch_position: Position3D,
        target_id: String,
    ) -> Self {
        // 初期速度は上方向（発射直後）
        let initial_velocity = Velocity3D::new(0.0, 0.0, 0.0);  // initializeで設定
        let initial_attitude = Attitude3D::new(0.0, 0.0, 0.0);
        
        Self {
            id,
            position: launch_position,
            velocity: initial_velocity,
            acceleration: Acceleration3D::new(0.0, 0.0, 0.0),
            target_id,
            status: AgentStatus::Active,
            initial_speed: 0.0,                     // initializeで設定
            max_speed: 0.0,                         // initializeで設定
            max_accel: 0.0,                         // initializeで設定
            max_turn_rate: 0.0,                     // initializeで設定
            intercept_radius: 0.0,                  // initializeで設定
            guidance_n: 0.0,                        // initializeで設定
            guidance_phase: GuidancePhase::Boost,
            endgame_threshold: 0.0,                 // initializeで設定
            miss_distance_history: Vec::new(),
            miss_increase_count: 0,
            endgame_miss_increase_ticks: 0,         // initializeで設定
            attitude: initial_attitude,
            turn_rate: 0.0,
            flight_time: 0.0,
            total_distance: 0.0,
            end_reason: None,
        }
    }

    /// True 3D比例航法による誘導計算
    /// 
    /// 真の3次元比例航法を使用して、ターゲットへの誘導加速度を計算します。
    /// 比例航法定数NとLOS（Line-of-Sight）角速度を使用して計算します。
    /// 
    /// # 引数
    /// 
    /// * `target_position` - ターゲットの現在位置
    /// 
    /// # 戻り値
    /// 
    /// 誘導に必要な加速度ベクトル
    pub fn calculate_proportional_navigation(&mut self, target_position: Position3D) -> Acceleration3D {
        let relative_position = target_position - self.position;
        let relative_distance = relative_position.magnitude();
        
        if relative_distance < 1e-6 {
            return Acceleration3D::new(0.0, 0.0, 0.0);
        }

        // 相対速度（ターゲットの速度は0と仮定、実際はターゲット速度も考慮が必要）
        let relative_velocity = self.velocity;
        
        // Line-of-Sight (LOS) 方向単位ベクトル
        let los_unit = Position3D::new(
            relative_position.x / relative_distance,
            relative_position.y / relative_distance,
            relative_position.z / relative_distance,
        );
        
        // 接近速度
        let closing_velocity = -(
            relative_velocity.x * los_unit.x +
            relative_velocity.y * los_unit.y +
            relative_velocity.z * los_unit.z
        );
        
        if closing_velocity <= 0.0 {
            // 離れている場合は直接追尾
            return self.calculate_direct_pursuit(target_position);
        }
        
        // LOS角速度の近似計算
        let los_rate_x = (relative_velocity.y * los_unit.z - relative_velocity.z * los_unit.y) / relative_distance;
        let los_rate_y = (relative_velocity.z * los_unit.x - relative_velocity.x * los_unit.z) / relative_distance;
        let los_rate_z = (relative_velocity.x * los_unit.y - relative_velocity.y * los_unit.x) / relative_distance;
        
        // 比例航法による必要加速度
        let accel_x = self.guidance_n * closing_velocity * los_rate_x;
        let accel_y = self.guidance_n * closing_velocity * los_rate_y;
        let accel_z = self.guidance_n * closing_velocity * los_rate_z;
        
        Acceleration3D::new(accel_x, accel_y, accel_z)
    }

    /// 直接追尾（緊急時用）
    /// 
    /// 比例航法が機能しない場合のフォールバックとして、
    /// ターゲットに直接向かう誘導を行います。
    /// 
    /// # 引数
    /// 
    /// * `target_position` - ターゲットの現在位置
    /// 
    /// # 戻り値
    /// 
    /// ターゲット方向への最大加速度ベクトル
    pub fn calculate_direct_pursuit(&self, target_position: Position3D) -> Acceleration3D {
        let direction = target_position - self.position;
        let distance = direction.magnitude();
        
        if distance < 1e-6 {
            return Acceleration3D::new(0.0, 0.0, 0.0);
        }
        
        // ターゲット方向への最大加速度
        let accel_magnitude = self.max_accel;
        Acceleration3D::new(
            (direction.x / distance) * accel_magnitude,
            (direction.y / distance) * accel_magnitude,
            (direction.z / distance) * accel_magnitude,
        )
    }

    /// 誘導フェーズの更新
    /// 
    /// ミサイルの現在状態に応じて誘導フェーズを遷移させます。
    /// Boost → Midcourse → Endgame の順で遷移します。
    /// 
    /// # 引数
    /// 
    /// * `target_position` - ターゲットの現在位置
    pub fn update_guidance_phase(&mut self, target_position: Position3D) {
        let distance = self.position.distance_3d(&target_position);
        
        match self.guidance_phase {
            GuidancePhase::Boost => {
                if self.flight_time > 2.0 {  // 2秒後にミッドコースへ
                    let previous_phase = self.guidance_phase;
                    self.guidance_phase = GuidancePhase::Midcourse;
                    
                    // フェーズ切替ログ（Boost → Midcourse）
                    debug!(
                        missile_id = %self.id,
                        target_id = %self.target_id,
                        position_x = self.position.x,
                        position_y = self.position.y,
                        position_z = self.position.z,
                        previous_phase = ?previous_phase,
                        current_phase = ?self.guidance_phase,
                        flight_time = self.flight_time,
                        target_distance = distance,
                        "MISSILE_PHASE_TRANSITION: ミサイル誘導フェーズが切り替わりました"
                    );
                }
            },
            GuidancePhase::Midcourse => {
                if distance <= self.endgame_threshold {
                    let previous_phase = self.guidance_phase;
                    self.guidance_phase = GuidancePhase::Endgame;
                    
                    // フェーズ切替ログ（Midcourse → Endgame）
                    debug!(
                        missile_id = %self.id,
                        target_id = %self.target_id,
                        position_x = self.position.x,
                        position_y = self.position.y,
                        position_z = self.position.z,
                        previous_phase = ?previous_phase,
                        current_phase = ?self.guidance_phase,
                        flight_time = self.flight_time,
                        target_distance = distance,
                        endgame_threshold = self.endgame_threshold,
                        "MISSILE_PHASE_TRANSITION: ミサイル誘導フェーズが切り替わりました（終盤フェーズへ）"
                    );
                }
            },
            GuidancePhase::Endgame => {
                // エンドゲームフェーズを維持
            }
        }
    }

    /// miss distanceの追跡と自爆判定
    /// 
    /// ターゲットへのmiss distanceを記録し、終盤フェーズで
    /// miss distanceが連続して増加した場合に自爆を判定します。
    /// 
    /// # 引数
    /// 
    /// * `target_position` - ターゲットの現在位置
    /// 
    /// # 戻り値
    /// 
    /// 自爆した場合はtrue、継続する場合はfalse
    pub fn track_miss_distance(&mut self, target_position: Position3D) -> bool {
        let miss_distance = self.calculate_miss_distance(target_position);
        
        // 履歴に追加
        self.miss_distance_history.push(miss_distance);
        
        // 履歴サイズ制限（メモリ管理）
        if self.miss_distance_history.len() > 10 {
            self.miss_distance_history.remove(0);
        }
        
        // 終盤フェーズでのmiss distance増加判定
        if self.guidance_phase == GuidancePhase::Endgame && self.miss_distance_history.len() >= 2 {
            let len = self.miss_distance_history.len();
            let current_miss = self.miss_distance_history[len - 1];
            let previous_miss = self.miss_distance_history[len - 2];
            
            if current_miss > previous_miss {
                self.miss_increase_count += 1;
            } else {
                self.miss_increase_count = 0;
            }
            
            // 連続増加回数が閾値を超えたら自爆
            if self.miss_increase_count >= self.endgame_miss_increase_ticks {
                self.status = AgentStatus::SelfDestruct;
                self.end_reason = Some(MissileEndReason::SelfDestruct);
                
                // 自爆ログ
                warn!(
                    missile_id = %self.id,
                    target_id = %self.target_id,
                    self_destruct_position_x = self.position.x,
                    self_destruct_position_y = self.position.y,
                    self_destruct_position_z = self.position.z,
                    flight_time = self.flight_time,
                    total_distance = self.total_distance,
                    miss_distance = current_miss,
                    miss_increase_count = self.miss_increase_count,
                    guidance_phase = ?self.guidance_phase,
                    "MISSILE_SELF_DESTRUCT: ミサイルが自爆しました（miss distance継続増加）"
                );
                
                return true;
            }
        }
        
        false
    }

    /// 姿勢の更新
    /// 
    /// 速度ベクトルから理想的な姿勢を計算し、
    /// 最大旋回レートで制限して姿勢を更新します。
    /// 
    /// # 引数
    /// 
    /// * `dt` - 時間ステップ（秒）
    pub fn update_attitude(&mut self, dt: f64) {
        let new_attitude = Attitude3D::from_velocity(&self.velocity);
        
        // 旋回レート制限を適用
        let pitch_change = math_utils::angle_difference(self.attitude.pitch, new_attitude.pitch);
        let yaw_change = math_utils::angle_difference(self.attitude.yaw, new_attitude.yaw);
        
        let max_angle_change = self.max_turn_rate * dt;
        
        let pitch_limited = if pitch_change.abs() > max_angle_change {
            self.attitude.pitch + pitch_change.signum() * max_angle_change
        } else {
            new_attitude.pitch
        };
        
        let yaw_limited = if yaw_change.abs() > max_angle_change {
            self.attitude.yaw + yaw_change.signum() * max_angle_change
        } else {
            new_attitude.yaw
        };
        
        self.attitude.pitch = pitch_limited;
        self.attitude.yaw = yaw_limited;
        self.turn_rate = (pitch_change.abs() + yaw_change.abs()) / dt;
    }

    /// 運動状態の更新（設計仕様の手順に従う）
    /// 
    /// 設計仕様に従った手順でミサイルの運動を更新します:
    /// 1. 誘導計算 → 2. 加速度飽和 → 3. 速度積分 → 4. 速度クランプ → 5. 位置更新 → 6. 姿勢更新
    /// 
    /// # 引数
    /// 
    /// * `dt` - 時間ステップ（秒）
    /// * `target_position` - ターゲットの現在位置
    pub fn update_kinematics(&mut self, dt: f64, target_position: Position3D) {
        // 1. 誘導計算
        self.acceleration = match self.guidance_phase {
            GuidancePhase::Boost => {
                // ブースト段階では上昇しつつターゲット方向へ
                let boost_accel = Acceleration3D::new(0.0, 0.0, self.max_accel * 0.5);
                let guidance_accel = self.calculate_proportional_navigation(target_position);
                boost_accel + Acceleration3D::new(guidance_accel.x * 0.5, guidance_accel.y * 0.5, 0.0)
            },
            _ => self.calculate_proportional_navigation(target_position),
        };
        
        // 2. 加速度ベクトル飽和
        self.acceleration = self.acceleration.clamp_magnitude(self.max_accel);
        
        // 3. 速度積分
        self.velocity = self.velocity + self.acceleration * dt;
        
        // 4. 速度上限クリップ
        self.velocity = self.velocity.clamp_magnitude(self.max_speed);
        
        // 5. 位置更新
        let previous_position = self.position;
        self.position = self.position + Position3D::new(
            self.velocity.x * dt,
            self.velocity.y * dt,
            self.velocity.z * dt,
        );
        
        // 高度制限適用
        self.position.z = self.position.z.clamp(0.0, 5000.0);
        
        // 6. 姿勢更新
        self.update_attitude(dt);
        
        // 統計更新
        self.flight_time += dt;
        self.total_distance += previous_position.distance_3d(&self.position);
    }

    /// 各種チェックの実行
    /// 
    /// ミサイルの状態をチェックし、必要に応じて終了条件を判定します。
    /// 領域外チェック、誘導フェーズ更新、miss distance追跡、衝突判定を行います。
    /// 
    /// # 引数
    /// 
    /// * `target_position` - ターゲットの現在位置
    pub fn perform_checks(&mut self, target_position: Position3D) {
        // 領域外チェック
        if !self.position.is_in_simulation_bounds() {
            self.status = AgentStatus::SelfDestruct;
            self.end_reason = Some(MissileEndReason::OutOfBounds);
            
            // 領域外ログ
            info!(
                missile_id = %self.id,
                target_id = %self.target_id,
                out_of_bounds_position_x = self.position.x,
                out_of_bounds_position_y = self.position.y,
                out_of_bounds_position_z = self.position.z,
                flight_time = self.flight_time,
                total_distance = self.total_distance,
                simulation_bounds_x = "±1,000,000m",
                simulation_bounds_y = "±1,000,000m",
                simulation_bounds_z = "0-5,000m",
                "MISSILE_OUT_OF_BOUNDS: ミサイルがシミュレーション領域外に出ました"
            );
            
            return;
        }
        
        // 誘導フェーズ更新
        self.update_guidance_phase(target_position);
        
        // miss distance追跡
        self.track_miss_distance(target_position);
        
        // 衝突判定
        if self.check_collision(target_position) {
            self.status = AgentStatus::Destroyed; // 命中
            self.end_reason = Some(MissileEndReason::Hit);
            
            // 命中ログ
            info!(
                missile_id = %self.id,
                target_id = %self.target_id,
                hit_position_x = self.position.x,
                hit_position_y = self.position.y,
                hit_position_z = self.position.z,
                target_position_x = target_position.x,
                target_position_y = target_position.y,
                target_position_z = target_position.z,
                flight_time = self.flight_time,
                total_distance = self.total_distance,
                intercept_distance = self.position.distance_3d(&target_position),
                "MISSILE_HIT: ミサイルがターゲットに命中しました"
            );
        }
    }
}

impl IAgent for Missile {
    fn initialize(&mut self, scenario_config: &crate::scenario::ScenarioConfig) {
        self.status = AgentStatus::Active;
        self.flight_time = 0.0;
        self.total_distance = 0.0;
        self.miss_distance_history.clear();
        self.miss_increase_count = 0;
        
        // ミサイル性能パラメータの設定
        let missile_kinematics = &scenario_config.missile_defaults.kinematics;
        self.initial_speed = missile_kinematics.initial_speed_mps;
        self.max_speed = missile_kinematics.max_speed_mps;
        self.max_accel = missile_kinematics.max_accel_mps2;
        self.max_turn_rate = missile_kinematics.max_turn_rate_deg_s;
        self.intercept_radius = missile_kinematics.intercept_radius_m;
        
        // 誘導設定の適用
        let guidance_config = &scenario_config.policy.missile_guidance;
        self.guidance_n = guidance_config.n;
        
        // 終盤設定の適用
        let endgame_factor = guidance_config.endgame_factor;
        self.endgame_miss_increase_ticks = guidance_config.endgame_miss_increase_ticks;
        
        // 終盤判定閾値を計算（迎撃距離の倍数）
        self.endgame_threshold = self.intercept_radius * endgame_factor;
        
        // 初期速度を上方向に設定（発射直後）
        self.velocity = Velocity3D::new(0.0, 0.0, self.initial_speed);
        self.attitude = Attitude3D::from_velocity(&self.velocity);
        
        // 発射ログ
        info!(
            missile_id = %self.id,
            target_id = %self.target_id,
            launch_position_x = self.position.x,
            launch_position_y = self.position.y,
            launch_position_z = self.position.z,
            initial_speed = self.initial_speed,
            max_speed = self.max_speed,
            guidance_n = self.guidance_n,
            intercept_radius = self.intercept_radius,
            "MISSILE_LAUNCHED: ミサイルが発射されました"
        );
        
        // 距離測定方式の設定
        let intercept_convention = &scenario_config.world.distance_conventions.intercept;
        match intercept_convention.as_str() {
            "3D" => {
                // 3D距離での迎撃判定（デフォルト）
            },
            "XY" => {
                // XY平面距離での迎撃判定
            },
            _ => {
                // デフォルト
            }
        }
    }

    fn tick(&mut self, dt: f64) {
        if self.status != AgentStatus::Active {
            return;
        }

        // ターゲット位置の取得が必要（実際のシミュレーションでは外部から提供）
        // ここではプレースホルダーとして原点を使用
        let target_position = Position3D::new(0.0, 0.0, 0.0);
        
        // 運動学更新
        self.update_kinematics(dt, target_position);
        
        // 各種チェック
        self.perform_checks(target_position);
    }

    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn is_active(&self) -> bool {
        self.status == AgentStatus::Active
    }
}

impl IMovable for Missile {
    fn move_agent(&mut self, dt: f64) {
        // tick()内のupdate_kinematics()で処理される
        if self.status == AgentStatus::Active {
            let target_pos = Position3D::new(0.0, 0.0, 0.0); // プレースホルダー
            self.update_kinematics(dt, target_pos);
        }
    }

    fn get_position(&self) -> Position3D {
        self.position
    }

    fn get_velocity(&self) -> Velocity3D {
        self.velocity
    }

    fn set_position(&mut self, position: Position3D) {
        self.position = position;
    }

    fn set_velocity(&mut self, velocity: Velocity3D) {
        self.velocity = velocity;
    }
}

impl IMissile for Missile {
    fn guidance(&mut self, target_position: Position3D, dt: f64) {
        self.update_kinematics(dt, target_position);
    }

    fn get_target_id(&self) -> String {
        self.target_id.clone()
    }

    fn get_intercept_radius(&self) -> f64 {
        self.intercept_radius
    }
}

impl ICollision for Missile {
    fn check_collision(&self, target_position: Position3D) -> bool {
        let distance = self.position.distance_3d(&target_position);
        distance <= self.intercept_radius
    }

    fn calculate_miss_distance(&self, target_position: Position3D) -> f64 {
        self.position.distance_3d(&target_position)
    }

    fn is_endgame_phase(&self, target_position: Position3D) -> bool {
        let distance = self.position.distance_3d(&target_position);
        distance <= self.endgame_threshold
    }
}