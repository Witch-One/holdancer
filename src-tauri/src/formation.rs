use serde::{de::Error, Deserialize, Serialize};
use serde_json::{json, Result};
use std::borrow::BorrowMut;
use std::str::Bytes;
use std::{
    fs::{self, read},
    io::Write,
    path::Path,
    sync::Mutex,
};
use tauri::path;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Position {
    x: f64,
    y: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Dancer {
    id: i32,
    name: String,
    position: Position,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Formation {
    id: i32,
    bgm: String,
    start_time: i32,
    end_time: i32,
    dancer: Vec<Dancer>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Data {
    count_id: i32,
    formation: Vec<Formation>,
}

lazy_static::lazy_static! {
    static ref DATA: Mutex<Data> = Mutex::new(load_data());
}

pub fn load_data() -> Data {
    let path = Path::new("data/data.json");
    if !path.exists() {
        // 如果不存在，则创建文件及其父目录
        fs::create_dir_all(path.parent().unwrap()).expect("Failed to create parent directory"); // 创建父目录
        fs::File::create(&path).expect("Failed to create file"); // 创建文件
        let new_data = Data {
            count_id: 0,
            formation: vec![Formation {
                id: 0,
                bgm: "".to_string(),
                start_time: 0,
                end_time: 500,
                dancer: vec![],
            }],
        };

        fs::write(path, serde_json::to_string(&new_data).unwrap())
            .expect("Failed to write to file"); // 写入空数组
        return new_data.clone();
    }
    println!("Path: {}", path.display());
    let data_str = read(path).expect("Failed to read file");
    serde_json::from_slice(&data_str).expect("Failed to parse JSON")
}

pub fn get_current_formation_by_timestamp(time: i32) -> Option<Formation> {
    let data = DATA.lock().expect("Failed to lock mutex");
    let formation = &data.formation;

    // 找到当前的 formation
    if let Some(shot) = formation
        .iter()
        .find(|shot| shot.start_time <= time && shot.end_time > time)
    {
        return Some(shot.clone());
    }

    // 找到前一个和后一个 formation
    let prev_shot = formation.iter().rev().find(|shot| shot.end_time < time);
    let next_shot = formation.iter().find(|shot| shot.start_time > time);
    // 如果找到了前一个和后一个，则计算插值
    if let (Some(prev), Some(next)) = (prev_shot, next_shot) {
        let progress = (time - prev.end_time) as f64 / (next.start_time - prev.end_time) as f64;

        let interpolated_dancers: Vec<Dancer> = prev
            .dancer
            .iter()
            .map(|dancer| {
                let prev_dancer = prev.dancer.iter().find(|d| d.id == dancer.id);
                let next_dancer = next.dancer.iter().find(|d| d.id == dancer.id);

                match (prev_dancer, next_dancer) {
                    (Some(prev_d), Some(next_d)) => Dancer {
                        id: dancer.id,
                        name: dancer.name.clone(),
                        position: Position {
                            x: prev_d.position.x
                                + (next_d.position.x - prev_d.position.x) * progress,
                            y: prev_d.position.y
                                + (next_d.position.y - prev_d.position.y) * progress,
                        },
                    },
                    _ => dancer.clone(), // 如果没有找到相应的舞者，保持原样
                }
            })
            .collect();

        return Some(Formation {
            id: prev.id, // 使用前一个 formation 的 ID
            start_time: prev.start_time,
            bgm: prev.bgm,
            end_time: next.end_time,
            dancer: interpolated_dancers,
        });
    }
    None
}

fn get_count_id() -> i32 {
    let mut data = DATA.lock().expect("Failed to lock mutex");
    data.count_id += 1;
    data.count_id
}

// Function to update the data and write back to JSON
pub fn update_formation(time: i32, new_formation: Formation) -> Result<()> {
    let mut data = DATA.lock().unwrap();

    // Find and update the formation
    for formation in data.formation.iter_mut() {
        if formation.start_time <= time && formation.end_time > time {
            *formation = new_formation.clone();
            break;
        }
    }

    let mut current_formation_position = data
        .formation
        .iter()
        .position(|shot| shot.start_time <= time && shot.end_time > time);

    match current_formation_position {
        Some(cur_formation) => {
            for formation in 0..cur_formation {
                data.formation[cur_formation] = new_formation.clone();
            }
        }
        None => {
            let mut last_formation = data.formation.iter().rev().position(|f| f.end_time < time);
            let last_formation = match last_formation {
                Some(last_formation) => last_formation,
                None => 0,
            };
            data.formation.splice(
                last_formation..last_formation,
                vec![Formation {
                    id: get_count_id(),
                    bgm: new_formation.bgm.clone(),
                    start_time: time,
                    end_time: time + 500,
                    dancer: new_formation.dancer.clone(),
                }],
            );
        }
    }

    // Write back to the JSON file
    let json_data = serde_json::to_string(&*data).expect("Failed to convert data to JSON");
    std::fs::write("data/data.json", json_data).expect("Failed to write data to file");

    Ok(())
}

pub fn add_formation() -> Result<()> {
    let mut data = DATA.lock().unwrap();
    let last_formation = data.formation.last();
    match last_formation {
        Some(formation) => {
            let new_formation = Formation {
                dancer: formation.dancer.clone(),
                bgm: formation.bgm.clone(),
                id: formation.id + 1,
                start_time: formation.end_time,
                end_time: formation.end_time + 500,
            };
            data.formation.push(new_formation);
        }
        None => {
            let new_formation = Formation {
                dancer: vec![],
                bgm: "".to_string(),
                id: 0,
                start_time: 0,
                end_time: 500,
            };
            data.formation.push(new_formation);
        }
    };
    let data_json = serde_json::to_string(&*data).expect("Failed to convert data to JSON");
    std::fs::write("data/data.json", data_json).expect("Failed to write data to file");
    Ok(())
}

pub fn add_dancer(formation_id: i32, dancer: Dancer) -> Result<()> {
    let mut data = DATA.lock().unwrap();
    let formation = data.formation.iter_mut().find(|f| f.id == formation_id);
    match formation {
        Some(formation) => {
            formation.dancer.push(dancer);
            Ok(())
        }
        None => Err(Error::custom(format!(
            "Formation with id {} not found",
            formation_id
        ))),
    }
}

pub fn add_new_dancer(formation_id: i32) -> Result<()> {
    let mut data = DATA.lock().unwrap();
    // 先克隆整个 formation 数据
    let formations_clone = data.formation.clone();
    let formation = match formations_clone.iter().find(|f| f.id == formation_id) {
        Some(formation) => formation,
        None => {
            return Err(Error::custom(format!(
                "Formation with id {} not found",
                formation_id
            )))
        }
    };

    // 创建一个绑定来存储克隆的数据
    let formation_clone = formation.clone();
    let last_dancer = formation_clone.dancer.last();

    let new_dancer = match last_dancer {
        Some(dancer) => Dancer {
            id: dancer.id + 1,
            name: (dancer.id + 1).to_string(),
            position: Position { x: 0.0, y: 0.0 },
        },
        None => Dancer {
            id: 0,
            name: "0".to_string(),
            position: Position { x: 0.0, y: 0.0 },
        },
    };

    // 在原始数据中找到并更新formation
    if let Some(formation) = data.formation.iter_mut().find(|f| f.id == formation_id) {
        formation.dancer.push(new_dancer);
    }

    Ok(())
}
