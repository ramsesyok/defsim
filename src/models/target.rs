use crate::models::{
    traits::{IAgent, IMovable},
    common::{Position3D, Velocity3D, AgentStatus},
};

/// 敵ターゲットエージェント
#[derive(Debug, Clone)]
pub struct Target {
    pub id: String,
    pub position: Position3D,
    pub velocity: Velocity3D,
    pub destination: Position3D, // 指揮所位置
    pub arrival_radius: f64,     // 到達範囲[m]
    pub endurance: u32,          // 耐久値
    pub max_endurance: u32,      // 最大耐久値
    pub status: AgentStatus,
    pub group_id: String,        // 所属グループID
    pub spawn_time: f64,         // 発射時刻[s]
    pub speed: f64,              // 速度[m/s]
}

impl Target {
    /// 新しいTargetインスタンスを作成（シンプルな初期化）
    pub fn new(
        id: String,
        start_position: Position3D,
        destination: Position3D,
        group_id: String,
    ) -> Self {
        Self {
            id,
            position: start_position,
            velocity: Velocity3D::new(0.0, 0.0, 0.0),   // initializeで設定
            destination,
            arrival_radius: 0.0,                        // initializeで設定
            endurance: 0,                               // initializeで設定
            max_endurance: 0,                           // initializeで設定
            status: AgentStatus::Inactive,              // spawn_timeまで非アクティブ
            group_id,
            spawn_time: 0.0,                            // initializeで設定
            speed: 0.0,                                 // initializeで設定
        }
    }

    /// ダメージを受ける
    pub fn take_damage(&mut self, damage: u32) {
        if self.status == AgentStatus::Active {
            self.endurance = self.endurance.saturating_sub(damage);
            if self.endurance == 0 {
                self.status = AgentStatus::Destroyed;
            }
        }
    }

    /// 到達判定をチェック
    pub fn check_arrival(&mut self) {
        if self.status == AgentStatus::Active {
            let distance_to_destination = self.position.distance_xy(&self.destination);
            if distance_to_destination <= self.arrival_radius {
                self.status = AgentStatus::Reached;
            }
        }
    }

    /// 領域外判定をチェック
    pub fn check_out_of_bounds(&mut self) {
        if self.status == AgentStatus::Active && !self.position.is_in_simulation_bounds() {
            self.status = AgentStatus::Inactive; // 領域外で消滅
        }
    }

    /// ターゲット固有のパラメータを設定
    pub fn set_parameters(
        &mut self,
        arrival_radius: f64,
        endurance: u32,
        spawn_time: f64,
        speed: f64,
    ) {
        self.arrival_radius = arrival_radius;
        self.endurance = endurance;
        self.max_endurance = endurance;
        self.spawn_time = spawn_time;
        self.speed = speed;
        
        // 目的地への方向ベクトルを計算して速度ベクトルを設定
        let direction = self.destination - self.position;
        let direction_magnitude = direction.magnitude();
        
        self.velocity = if direction_magnitude > 0.0 {
            Velocity3D::new(
                (direction.x / direction_magnitude) * speed,
                (direction.y / direction_magnitude) * speed,
                (direction.z / direction_magnitude) * speed,
            )
        } else {
            Velocity3D::new(0.0, 0.0, 0.0)
        };
    }

    /// スポーン判定
    pub fn check_spawn(&mut self, current_time: f64) {
        if self.status == AgentStatus::Inactive && current_time >= self.spawn_time {
            self.status = AgentStatus::Active;
        }
    }


    /// 到達予想時刻を計算（Tgo計算用）
    pub fn calculate_time_to_go(&self) -> f64 {
        if self.status != AgentStatus::Active {
            return f64::INFINITY;
        }
        
        let distance_xy = self.position.distance_xy(&self.destination);
        let remaining_distance = (distance_xy - self.arrival_radius).max(0.0);
        
        if self.speed > 0.0 {
            remaining_distance / self.speed
        } else {
            f64::INFINITY
        }
    }
}

impl IAgent for Target {
    fn initialize(&mut self, scenario_config: &crate::scenario::ScenarioConfig) {
        // 距離測定方式の設定
        let distance_convention = &scenario_config.world.distance_conventions.breakthrough;
        match distance_convention.as_str() {
            "XY" => {
                // XY平面での突破判定（デフォルト）
            },
            "3D" => {
                // 3D距離での突破判定
            },
            _ => {
                // デフォルト
            }
        }
        
        // 個別のターゲットパラメータは既にコンストラクタで設定済み
        // ここではグローバルな設定値のみ適用
    }

    fn tick(&mut self, dt: f64) {
        // 現在時刻を計算（簡略化のため、dtの累積として扱う）
        static mut CURRENT_TIME: f64 = 0.0;
        unsafe {
            CURRENT_TIME += dt;
            self.check_spawn(CURRENT_TIME);
        }

        // アクティブな場合のみ処理
        if self.status == AgentStatus::Active {
            // 移動処理
            self.move_agent(dt);
            
            // 到達判定
            self.check_arrival();
            
            // 領域外判定
            self.check_out_of_bounds();
        }
    }

    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn is_active(&self) -> bool {
        self.status == AgentStatus::Active
    }
}

impl IMovable for Target {
    fn move_agent(&mut self, dt: f64) {
        if self.status == AgentStatus::Active {
            // 等速直線運動
            self.position = self.position + Position3D::new(
                self.velocity.x * dt,
                self.velocity.y * dt,
                self.velocity.z * dt,
            );
            
            // 高度制限を適用
            self.position.z = self.position.z.clamp(0.0, 5000.0);
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

/// 敵グループの配置パターンを生成するヘルパー構造体
pub struct TargetGroup {
    pub id: String,
    pub center_position: Position3D,
    pub count: u32,
    pub ring_spacing: f64,
    pub start_angle: f64,
    pub ring_half_offset: bool,
    pub endurance: u32,
    pub spawn_time: f64,
    pub speed: f64,
    pub destination: Position3D,
    pub arrival_radius: f64,
}

impl TargetGroup {
    /// グループ内のターゲット配置位置を計算
    pub fn generate_positions(&self) -> Vec<Position3D> {
        let mut positions = Vec::new();
        let mut remaining_count = self.count as usize;
        let mut ring_index = 1;
        let mut _total_placed = 0;

        while remaining_count > 0 {
            let ring_radius = ring_index as f64 * self.ring_spacing;
            let circumference = 2.0 * std::f64::consts::PI * ring_radius;
            let max_positions_in_ring = if ring_index == 1 { 
                1 // 中心に1個
            } else {
                (circumference / self.ring_spacing) as usize
            };
            
            let positions_in_ring = remaining_count.min(max_positions_in_ring);
            
            if ring_index == 1 && positions_in_ring > 0 {
                // 中心位置
                positions.push(self.center_position);
                remaining_count -= 1;
                _total_placed += 1;
            } else if ring_index > 1 {
                // リング配置
                let angle_step = 360.0 / positions_in_ring as f64;
                let angle_offset = if self.ring_half_offset && ring_index > 2 {
                    angle_step / 2.0 // 外側リングは半角オフセット
                } else {
                    0.0
                };
                
                for i in 0..positions_in_ring {
                    let angle = self.start_angle + (i as f64 * angle_step) + angle_offset;
                    let angle_rad = angle * std::f64::consts::PI / 180.0;
                    
                    let x = self.center_position.x + ring_radius * angle_rad.cos();
                    let y = self.center_position.y + ring_radius * angle_rad.sin();
                    let z = self.center_position.z;
                    
                    positions.push(Position3D::new(x, y, z));
                }
                
                remaining_count -= positions_in_ring;
                _total_placed += positions_in_ring;
            }
            
            ring_index += 1;
        }

        positions
    }

    /// グループ内の全ターゲットを生成
    pub fn generate_targets(&self) -> Vec<Target> {
        let positions = self.generate_positions();
        let mut targets = Vec::new();

        for (index, position) in positions.iter().enumerate() {
            let target_id = format!("{}_T{:03}", self.id, index + 1);
            let mut target = Target::new(
                target_id,
                *position,
                self.destination,
                self.id.clone(),
            );
            
            // パラメータを設定
            target.set_parameters(
                self.arrival_radius,
                self.endurance,
                self.spawn_time,
                self.speed,
            );
            
            targets.push(target);
        }

        targets
    }
}