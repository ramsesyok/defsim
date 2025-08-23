mod models;

use models::*;

fn main() {
    println!("防衛シミュレーション (Defense Simulation) - defsim");
    
    // エージェントモデルのテスト用サンプル
    test_agent_models();
}

fn test_agent_models() {
    println!("\n=== エージェントモデルのテスト ===");
    
    // 指揮所の作成
    let command_post_pos = Position3D::new(800000.0, -800000.0, 0.0);
    let command_post = CommandPost::new(
        "CP001".to_string(),
        command_post_pos,
        20000.0, // 到達範囲
    );
    println!("指揮所が作成されました: {}", command_post.get_id());
    
    // センサーの作成
    let sensor_pos = Position3D::new(750000.0, -900000.0, 50.0);
    let sensor = Sensor::new(
        "S001".to_string(),
        sensor_pos,
        180000.0, // 探知範囲
    );
    println!("センサーが作成されました: {}", sensor.get_id());
    
    // ランチャーの作成
    let launcher_pos = Position3D::new(780000.0, -850000.0, 20.0);
    let launcher = Launcher::new(
        "L001".to_string(),
        launcher_pos,
        4,      // 最大ミサイル数
        5.0,    // クールダウン時間
        300.0,  // 初速
        1200.0, // 最大速度
        80.0,   // 最大加速度
        40.0,   // 最大旋回レート
        50.0,   // 迎撃判定距離
    );
    println!("ランチャーが作成されました: {}", launcher.get_id());
    
    // ターゲットグループの作成
    let group_center = Position3D::new(-800000.0, 600000.0, 3000.0);
    let target_group = TargetGroup {
        id: "G001_wave1".to_string(),
        center_position: group_center,
        count: 12,
        ring_spacing: 1500.0,
        start_angle: 0.0,
        ring_half_offset: true,
        endurance: 2,
        spawn_time: 120.0,
        speed: 200.0,
        destination: command_post_pos,
        arrival_radius: 20000.0,
    };
    
    let targets = target_group.generate_targets();
    println!("ターゲットグループが作成されました: {} 機の敵", targets.len());
    
    // ミサイルの作成テスト
    let missile = Missile::new(
        "L001_M001".to_string(),
        launcher_pos,
        "G001_wave1_T001".to_string(),
        300.0,  // 初速
        1200.0, // 最大速度
        80.0,   // 最大加速度
        40.0,   // 最大旋回レート
        50.0,   // 迎撃判定距離
    );
    println!("ミサイルが作成されました: {}", missile.get_id());
    
    println!("\n全てのエージェントモデルが正常に作成されました！");
}
