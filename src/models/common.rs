use std::ops::{Add, Sub, Mul};

/// 3次元位置を表す構造体
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position3D {
    pub x: f64, // m
    pub y: f64, // m
    pub z: f64, // m (altitude)
}

impl Position3D {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { 
            x, 
            y, 
            z: z.clamp(0.0, 5000.0) // 高度範囲制限
        }
    }

    /// XY平面での2次元距離を計算
    pub fn distance_xy(&self, other: &Position3D) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    /// 3次元距離を計算
    pub fn distance_3d(&self, other: &Position3D) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2) + (self.z - other.z).powi(2)).sqrt()
    }

    /// ベクトルの長さ（原点からの距離）
    pub fn magnitude(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }

    /// XY平面での角度を計算（度）
    pub fn angle_xy(&self) -> f64 {
        self.y.atan2(self.x).to_degrees()
    }

    /// シミュレーション領域内かどうかを判定
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Velocity3D {
    pub x: f64, // m/s
    pub y: f64, // m/s
    pub z: f64, // m/s
}

impl Velocity3D {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// 速度ベクトルの大きさ
    pub fn magnitude(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }

    /// 速度ベクトルを正規化
    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag > 0.0 {
            Self::new(self.x / mag, self.y / mag, self.z / mag)
        } else {
            *self
        }
    }

    /// XY平面での速度の大きさ
    pub fn magnitude_xy(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    /// 速度制限（最大速度でクリップ）
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Acceleration3D {
    pub x: f64, // m/s²
    pub y: f64, // m/s²
    pub z: f64, // m/s²
}

impl Acceleration3D {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// 加速度ベクトルの大きさ
    pub fn magnitude(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }

    /// 加速度制限（最大加速度でクリップ）
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AgentStatus {
    Active,      // アクティブ
    Destroyed,   // 撃破
    Reached,     // 目標到達（敵の場合）
    SelfDestruct, // 自爆（ミサイルの場合）
    Inactive,    // 非アクティブ
}

/// エージェントの種類を表す列挙型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AgentType {
    Target,
    CommandPost,
    Sensor,
    Launcher,
    Missile,
}

/// シミュレーション定数
pub struct SimulationConstants {
    /// シミュレーション領域の境界
    pub region_bounds: (f64, f64, f64, f64), // (x_min, x_max, y_min, y_max)
    /// 高度範囲
    pub altitude_range: (f64, f64), // (z_min, z_max)
    /// デフォルト値
    pub default_dt: f64, // デフォルトΔt
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
pub mod math_utils {
    /// 度をラジアンに変換
    pub fn deg_to_rad(degrees: f64) -> f64 {
        degrees * std::f64::consts::PI / 180.0
    }

    /// ラジアンを度に変換
    pub fn rad_to_deg(radians: f64) -> f64 {
        radians * 180.0 / std::f64::consts::PI
    }

    /// 角度を-180度〜180度の範囲に正規化
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
    pub fn angle_difference(angle1_deg: f64, angle2_deg: f64) -> f64 {
        normalize_angle(angle2_deg - angle1_deg)
    }
}