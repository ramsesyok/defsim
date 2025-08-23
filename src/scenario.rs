use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;

/// シナリオメタデータ
#[derive(Debug, Deserialize, Serialize)]
pub struct ScenarioMeta {
    pub version: String,
    pub name: String,
    pub description: String,
}

/// シミュレーション設定
#[derive(Debug, Deserialize, Serialize)]
pub struct SimulationConfig {
    pub dt_s: f64,
    pub t_max_s: f64,
    pub seed: u64,
}

/// 世界設定
#[derive(Debug, Deserialize, Serialize)]
pub struct WorldConfig {
    pub region_rect: RegionRect,
    pub z_limits_m: [f64; 2],
    pub distance_conventions: DistanceConventions,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RegionRect {
    pub xmin_m: f64,
    pub xmax_m: f64,
    pub ymin_m: f64,
    pub ymax_m: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DistanceConventions {
    pub breakthrough: String,
    pub sensor: String,
    pub launcher_selection: String,
    pub intercept: String,
}

/// 指揮所設定
#[derive(Debug, Deserialize, Serialize)]
pub struct CommandPostConfig {
    pub position: Position2D,
    pub arrival_radius_m: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Position2D {
    pub x_m: f64,
    pub y_m: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Position3D {
    pub x_m: f64,
    pub y_m: f64,
    pub z_m: f64,
}

/// 戦術ポリシー設定
#[derive(Debug, Deserialize, Serialize)]
pub struct PolicyConfig {
    pub tgo_definition: String,
    pub tie_breakers: Vec<String>,
    pub launcher_selection_order: Vec<String>,
    pub launcher_initially_cooled: bool,
    pub angle_reference: AngleReference,
    pub missile_guidance: MissileGuidanceConfig,
    pub missile_kinematics_defaults: MissileKinematics,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AngleReference {
    pub zero_deg_axis: String,
    pub rotation: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MissileGuidanceConfig {
    pub r#type: String, // "type"はRustのキーワードなのでr#でエスケープ
    #[serde(rename = "N")]
    pub n: f64,
    pub endgame_factor: f64,
    pub endgame_miss_increase_ticks: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MissileKinematics {
    pub initial_speed_mps: f64,
    pub max_speed_mps: f64,
    pub max_accel_mps2: f64,
    pub max_turn_rate_deg_s: f64,
    pub intercept_radius_m: f64,
}

/// 友軍設定
#[derive(Debug, Deserialize, Serialize)]
pub struct FriendlyForcesConfig {
    pub deploy_rect_xy: Option<RegionRect>,
    pub sensors: Vec<SensorConfig>,
    pub launchers: Vec<LauncherConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SensorConfig {
    pub id: String,
    pub pos: Position3D,
    pub range_m: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LauncherConfig {
    pub id: String,
    pub pos: Position3D,
    pub missiles_loaded: u32,
    pub cooldown_s: f64,
}

/// 敵軍設定
#[derive(Debug, Deserialize, Serialize)]
pub struct EnemyForcesConfig {
    pub spawn_rect_xy: RegionRect,
    pub groups: Vec<EnemyGroupConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EnemyGroupConfig {
    pub id: String,
    pub spawn_time_s: f64,
    pub center_xy: Position2D,
    pub z_m: f64,
    pub count: u32,
    pub ring_spacing_m: f64,
    pub start_angle_deg: f64,
    pub ring_half_offset: bool,
    pub endurance_pt: u32,
    pub speed_mps: f64,
}

/// 完全なシナリオ設定
#[derive(Debug, Deserialize, Serialize)]
pub struct ScenarioConfig {
    pub meta: ScenarioMeta,
    pub sim: SimulationConfig,
    pub world: WorldConfig,
    pub command_post: CommandPostConfig,
    pub policy: PolicyConfig,
    pub friendly_forces: FriendlyForcesConfig,
    pub enemy_forces: EnemyForcesConfig,
    pub missile_defaults: MissileDefaults,
}

/// ミサイルデフォルト設定
#[derive(Debug, Deserialize, Serialize)]
pub struct MissileDefaults {
    pub kinematics: MissileKinematics,
}

impl ScenarioConfig {
    /// YAMLファイルからシナリオ設定を読み込み
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ScenarioError> {
        let path = path.as_ref();
        
        // ファイル存在チェック
        if !path.exists() {
            return Err(ScenarioError::FileNotFound(path.to_path_buf()));
        }
        
        // ファイル読み込み
        let contents = fs::read_to_string(path)
            .map_err(|e| ScenarioError::IoError(path.to_path_buf(), e))?;
        
        // YAML解析
        let config: ScenarioConfig = serde_yaml::from_str(&contents)
            .map_err(|e| ScenarioError::ParseError(path.to_path_buf(), e))?;
        
        // 基本的な検証
        config.validate()?;
        
        Ok(config)
    }
    
    /// 設定の基本的な検証
    pub fn validate(&self) -> Result<(), ScenarioError> {
        // 時間設定の検証
        if self.sim.dt_s <= 0.0 {
            return Err(ScenarioError::ValidationError("dt_s must be positive".to_string()));
        }
        if self.sim.t_max_s <= 0.0 {
            return Err(ScenarioError::ValidationError("t_max_s must be positive".to_string()));
        }
        
        // 座標範囲の検証
        let region = &self.world.region_rect;
        if region.xmin_m >= region.xmax_m || region.ymin_m >= region.ymax_m {
            return Err(ScenarioError::ValidationError("Invalid region bounds".to_string()));
        }
        
        // 高度範囲の検証
        let z_limits = &self.world.z_limits_m;
        if z_limits[0] >= z_limits[1] || z_limits[0] < 0.0 {
            return Err(ScenarioError::ValidationError("Invalid z_limits".to_string()));
        }
        
        // 指揮所位置の検証
        let cp_pos = &self.command_post.position;
        if !self.is_position_in_bounds(cp_pos.x_m, cp_pos.y_m) {
            return Err(ScenarioError::ValidationError("Command post outside region bounds".to_string()));
        }
        
        // 敵グループのスポーン時刻検証
        for group in &self.enemy_forces.groups {
            if group.spawn_time_s >= self.sim.t_max_s {
                return Err(ScenarioError::ValidationError(
                    format!("Group {} spawn time {} >= simulation time {}", 
                            group.id, group.spawn_time_s, self.sim.t_max_s)
                ));
            }
        }
        
        Ok(())
    }
    
    /// 位置が領域内かどうかをチェック
    fn is_position_in_bounds(&self, x: f64, y: f64) -> bool {
        let region = &self.world.region_rect;
        x >= region.xmin_m && x <= region.xmax_m &&
        y >= region.ymin_m && y <= region.ymax_m
    }
    
    /// シナリオの概要を表示
    pub fn print_summary(&self) {
        println!("=== シナリオ情報 ===");
        println!("名前: {}", self.meta.name);
        println!("説明: {}", self.meta.description);
        println!("バージョン: {}", self.meta.version);
        println!();
        
        println!("=== シミュレーション設定 ===");
        println!("時間刻み: {:.3}秒", self.sim.dt_s);
        println!("最大時間: {:.1}秒 ({:.1}分)", self.sim.t_max_s, self.sim.t_max_s / 60.0);
        println!("シード値: {}", self.sim.seed);
        println!();
        
        println!("=== 友軍戦力 ===");
        println!("センサー: {}基", self.friendly_forces.sensors.len());
        println!("ランチャー: {}基", self.friendly_forces.launchers.len());
        let total_missiles: u32 = self.friendly_forces.launchers.iter().map(|l| l.missiles_loaded).sum();
        println!("総ミサイル数: {}発", total_missiles);
        println!();
        
        println!("=== 敵軍戦力 ===");
        println!("敵グループ数: {}", self.enemy_forces.groups.len());
        let total_enemies: u32 = self.enemy_forces.groups.iter().map(|g| g.count).sum();
        println!("総敵機数: {}機", total_enemies);
        
        for group in &self.enemy_forces.groups {
            println!("  {}: {}機 (出現時刻: {:.1}秒)", group.id, group.count, group.spawn_time_s);
        }
    }
}

/// シナリオ読み込みエラー
#[derive(Debug)]
pub enum ScenarioError {
    FileNotFound(std::path::PathBuf),
    IoError(std::path::PathBuf, std::io::Error),
    ParseError(std::path::PathBuf, serde_yaml::Error),
    ValidationError(String),
}

impl std::fmt::Display for ScenarioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScenarioError::FileNotFound(path) => {
                write!(f, "シナリオファイルが見つかりません: {}", path.display())
            }
            ScenarioError::IoError(path, err) => {
                write!(f, "ファイル読み込みエラー {}: {}", path.display(), err)
            }
            ScenarioError::ParseError(path, err) => {
                write!(f, "YAML解析エラー {}: {}", path.display(), err)
            }
            ScenarioError::ValidationError(msg) => {
                write!(f, "設定検証エラー: {}", msg)
            }
        }
    }
}

impl std::error::Error for ScenarioError {}