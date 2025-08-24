use std::ops::{Add, Sub, Mul};

/// 3次元位置を表す構造体
/// 
/// シミュレーション空間内の位置を表現します。
/// 座標系: X軸（右方向）、Y軸（上方向）、Z軸（高度）
/// 単位: メートル（m）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position3D {
    /// X座標（メートル）
    pub x: f64,
    /// Y座標（メートル）
    pub y: f64,
    /// Z座標（高度、メートル）
    pub z: f64,
}

impl Position3D {
    /// 新しい3次元位置を作成
    /// 
    /// # 引数
    /// 
    /// * `x` - X座標（メートル）
    /// * `y` - Y座標（メートル）
    /// * `z` - Z座標（高度、メートル）。0-5000mの範囲でクランプされます
    /// 
    /// # 戻り値
    /// 
    /// 新しいPosition3Dインスタンス
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { 
            x, 
            y, 
            z: z.clamp(0.0, 5000.0) // 高度範囲制限
        }
    }

    /// XY平面での2次元距離を計算
    /// 
    /// # 引数
    /// 
    /// * `other` - 距離を測定する対象の位置
    /// 
    /// # 戻り値
    /// 
    /// XY平面での距離（メートル）
    pub fn distance_xy(&self, other: &Position3D) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    /// 3次元距離を計算
    /// 
    /// # 引数
    /// 
    /// * `other` - 距離を測定する対象の位置
    /// 
    /// # 戻り値
    /// 
    /// 3次元空間での距離（メートル）
    pub fn distance_3d(&self, other: &Position3D) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2) + (self.z - other.z).powi(2)).sqrt()
    }

    /// ベクトルの長さ（原点からの距離）
    /// 
    /// # 戻り値
    /// 
    /// 原点(0,0,0)からのユークリッド距離（メートル）
    pub fn magnitude(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }

    /// XY平面での角度を計算（度）
    /// 
    /// X軸の正の方向を0度とし、反時計回りを正とする角度を計算します。
    /// 
    /// # 戻り値
    /// 
    /// 角度（度）、-180度〜180度の範囲
    pub fn angle_xy(&self) -> f64 {
        self.y.atan2(self.x).to_degrees()
    }

    /// シミュレーション領域内かどうかを判定
    /// 
    /// シミュレーション領域（±100万m四方、高度0-5000m）内にあるかを確認します。
    /// 
    /// # 戻り値
    /// 
    /// 領域内にある場合はtrue、範囲外の場合はfalse
    pub fn is_in_simulation_bounds(&self) -> bool {
        self.x >= -1_000_000.0 && self.x <= 1_000_000.0 &&
        self.y >= -1_000_000.0 && self.y <= 1_000_000.0 &&
        self.z >= 0.0 && self.z <= 5_000.0
    }
}

impl Add for Position3D {
    type Output = Self;
    
    fn add(self, other: Self) -> Self::Output {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl Sub for Position3D {
    type Output = Self;
    
    fn sub(self, other: Self) -> Self::Output {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

/// 3次元速度を表す構造体
/// 
/// シミュレーション空間内の速度ベクトルを表現します。
/// 単位: メートル毎秒（m/s）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Velocity3D {
    /// X方向の速度成分（m/s）
    pub x: f64,
    /// Y方向の速度成分（m/s）
    pub y: f64,
    /// Z方向の速度成分（m/s）
    pub z: f64,
}

impl Velocity3D {
    /// 新しい3次元速度を作成
    /// 
    /// # 引数
    /// 
    /// * `x` - X方向の速度成分（m/s）
    /// * `y` - Y方向の速度成分（m/s）
    /// * `z` - Z方向の速度成分（m/s）
    /// 
    /// # 戻り値
    /// 
    /// 新しいVelocity3Dインスタンス
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// 速度ベクトルの大きさ
    /// 
    /// # 戻り値
    /// 
    /// 速度ベクトルのユークリッドノルム（m/s）
    pub fn magnitude(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }

    /// 速度ベクトルを正規化
    /// 
    /// 速度ベクトルの大きさを1に正規化します。ゼロベクトルの場合はそのまま返します。
    /// 
    /// # 戻り値
    /// 
    /// 正規化された速度ベクトル
    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag > 0.0 {
            Self::new(self.x / mag, self.y / mag, self.z / mag)
        } else {
            *self
        }
    }

    /// XY平面での速度の大きさ
    /// 
    /// # 戻り値
    /// 
    /// XY平面での速度ベクトルの大きさ（m/s）
    pub fn magnitude_xy(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    /// 速度制限（最大速度でクリップ）
    /// 
    /// 速度ベクトルの大きさを最大速度で制限します。
    /// 
    /// # 引数
    /// 
    /// * `max_speed` - 最大速度（m/s）
    /// 
    /// # 戻り値
    /// 
    /// 制限された速度ベクトル
    pub fn clamp_magnitude(&self, max_speed: f64) -> Self {
        let mag = self.magnitude();
        if mag > max_speed {
            let factor = max_speed / mag;
            Self::new(self.x * factor, self.y * factor, self.z * factor)
        } else {
            *self
        }
    }
}

impl Add for Velocity3D {
    type Output = Self;
    
    fn add(self, other: Self) -> Self::Output {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl Mul<f64> for Velocity3D {
    type Output = Self;
    
    fn mul(self, scalar: f64) -> Self::Output {
        Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

// Velocity3D + Acceleration3D*dt の演算を可能にする
impl Add<Acceleration3D> for Velocity3D {
    type Output = Self;
    
    fn add(self, acceleration: Acceleration3D) -> Self::Output {
        Self::new(self.x + acceleration.x, self.y + acceleration.y, self.z + acceleration.z)
    }
}

/// 3次元加速度を表す構造体
/// 
/// シミュレーション空間内の加速度ベクトルを表現します。
/// 単位: メートル毎秒の2乗（m/s²）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Acceleration3D {
    /// X方向の加速度成分（m/s²）
    pub x: f64,
    /// Y方向の加速度成分（m/s²）
    pub y: f64,
    /// Z方向の加速度成分（m/s²）
    pub z: f64,
}

impl Acceleration3D {
    /// 新しい3次元加速度を作成
    /// 
    /// # 引数
    /// 
    /// * `x` - X方向の加速度成分（m/s²）
    /// * `y` - Y方向の加速度成分（m/s²）
    /// * `z` - Z方向の加速度成分（m/s²）
    /// 
    /// # 戻り値
    /// 
    /// 新しいAcceleration3Dインスタンス
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// 加速度ベクトルの大きさ
    /// 
    /// # 戻り値
    /// 
    /// 加速度ベクトルのユークリッドノルム（m/s²）
    pub fn magnitude(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }

    /// 加速度制限（最大加速度でクリップ）
    /// 
    /// 加速度ベクトルの大きさを最大加速度で制限します。
    /// 
    /// # 引数
    /// 
    /// * `max_accel` - 最大加速度（m/s²）
    /// 
    /// # 戻り値
    /// 
    /// 制限された加速度ベクトル
    pub fn clamp_magnitude(&self, max_accel: f64) -> Self {
        let mag = self.magnitude();
        if mag > max_accel {
            let factor = max_accel / mag;
            Self::new(self.x * factor, self.y * factor, self.z * factor)
        } else {
            *self
        }
    }
}

impl Add for Acceleration3D {
    type Output = Self;
    
    fn add(self, other: Self) -> Self::Output {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl Mul<f64> for Acceleration3D {
    type Output = Self;
    
    fn mul(self, scalar: f64) -> Self::Output {
        Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

/// エージェントの状態を表す列挙型
/// 
/// シミュレーション内のすべてのエージェントが取り得る可能性がある状態です。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AgentStatus {
    /// アクティブ状態（正常動作中）
    Active,
    /// 撃破された状態
    Destroyed,
    /// 目標到達状態（ターゲットが指揮所に到達）
    Reached,
    /// 自爆状態（ミサイルの終端処理）
    SelfDestruct,
    /// 非アクティブ状態（停止中）
    Inactive,
}

/// エージェントの種類を表す列挙型
/// 
/// シミュレーション内の各エージェントのタイプを区別します。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AgentType {
    /// ターゲット（敵）エージェント
    Target,
    /// 指揮所エージェント
    CommandPost,
    /// センサーエージェント
    Sensor,
    /// ランチャーエージェント
    Launcher,
    /// ミサイルエージェント
    Missile,
}

/// シミュレーション定数
/// 
/// シミュレーション全体で使用される基本的な定数値を管理します。
pub struct SimulationConstants {
    /// シミュレーション領域の境界 (x_min, x_max, y_min, y_max)
    pub region_bounds: (f64, f64, f64, f64),
    /// 高度範囲 (z_min, z_max)
    pub altitude_range: (f64, f64),
    /// デフォルトの時間ステップΔt（秒）
    pub default_dt: f64,
}

impl Default for SimulationConstants {
    fn default() -> Self {
        Self {
            region_bounds: (-1_000_000.0, 1_000_000.0, -1_000_000.0, 1_000_000.0),
            altitude_range: (0.0, 5_000.0),
            default_dt: 0.1,
        }
    }
}

/// 数学ユーティリティ関数
/// 
/// シミュレーションで使用される数学的なユーティリティ関数です。
pub mod math_utils {
    /// 度をラジアンに変換
    /// 
    /// # 引数
    /// 
    /// * `degrees` - 角度（度）
    /// 
    /// # 戻り値
    /// 
    /// 角度（ラジアン）
    pub fn deg_to_rad(degrees: f64) -> f64 {
        degrees * std::f64::consts::PI / 180.0
    }

    /// ラジアンを度に変換
    /// 
    /// # 引数
    /// 
    /// * `radians` - 角度（ラジアン）
    /// 
    /// # 戻り値
    /// 
    /// 角度（度）
    pub fn rad_to_deg(radians: f64) -> f64 {
        radians * 180.0 / std::f64::consts::PI
    }

    /// 角度を-180度〜180度の範囲に正規化
    /// 
    /// # 引数
    /// 
    /// * `angle_deg` - 正規化する角度（度）
    /// 
    /// # 戻り値
    /// 
    /// -180度〜180度の範囲に正規化された角度
    pub fn normalize_angle(angle_deg: f64) -> f64 {
        let mut normalized = angle_deg % 360.0;
        if normalized > 180.0 {
            normalized -= 360.0;
        } else if normalized <= -180.0 {
            normalized += 360.0;
        }
        normalized
    }

    /// 2つの角度の差を計算（-180度〜180度の範囲）
    /// 
    /// # 引数
    /// 
    /// * `angle1_deg` - 基準角度（度）
    /// * `angle2_deg` - 目標角度（度）
    /// 
    /// # 戻り値
    /// 
    /// 角度差 (angle2 - angle1)、-180度〜180度の範囲
    pub fn angle_difference(angle1_deg: f64, angle2_deg: f64) -> f64 {
        normalize_angle(angle2_deg - angle1_deg)
    }
}