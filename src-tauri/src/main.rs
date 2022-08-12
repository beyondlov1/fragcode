#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use serde_json::{Value};
use serde_json::json;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use rusqlite::{ Connection};
use std::path::{Path, PathBuf};
use std::fs;
use tauri::{SystemTray,SystemTrayMenu, SystemTraySubmenu,SystemTrayEvent, SystemTrayMenuItem, CustomMenuItem,AppHandle,Manager,GlobalShortcutManager};


#[derive(Serialize, Deserialize, Debug)]
struct FragCode {
    id: i32,
    abbr: String,
    code: String,
}

#[derive(Debug)]
struct TableField {
    id: i32,
    table_name: String,
    field_name: String,
    field_type: i32,
    field_default: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Count {
    cnt: i32
}

fn connect()-> Connection{
  let home = dirs::home_dir().unwrap();
  let mut dir = PathBuf::from(home);
  dir.push(".fragcode");
  let mut path = PathBuf::from(&dir);
  path.push("my.db");
  if !&dir.as_path().exists(){
      fs::create_dir_all(&dir).unwrap();
  }
  Connection::open(path).unwrap()
}

fn insert(conn: &Connection,v:&Vec<FragCode>) -> rusqlite::Result<()>{
  let mut vtup = Vec::new();
  for item in v{
    vtup.push((&item.abbr, &item.code));
  }
  for tup in vtup{
    conn.execute(
      "INSERT INTO t_frag_code (abbr, code) VALUES (?1, ?2)",
      tup,
    )?;
  }
  Ok(())
}

fn delete(conn: &Connection,id:i32) -> rusqlite::Result<()>{
  conn.execute(
    "DELETE FROM t_frag_code where id = ?1",
    [id],
  )?;
  Ok(())
}

fn createtable(conn: &Connection) -> rusqlite::Result<()>{
    conn.execute(
        "CREATE TABLE IF NOT EXISTS t_frag_code (
            id   INTEGER PRIMARY KEY,
            abbr TEXT NOT NULL DEFAULT '',
            code TEXT NOT NULL DEFAULT ''
        )",
        (), // empty list of parameters.
    )?;

    let mut stmt = conn.prepare("SELECT count(name) FROM sqlite_master where tbl_name = 't_frag_code' and type='table' and sql like '%access_cnt%'")?;
    let mut rs = stmt.query_map([], |row| {
      Ok(Count{
        cnt: row.get(0)?
      })
    })?;

    let first = rs.next().unwrap()?;
    let count = first.cnt;
    if count == 0{
      conn.execute(
        "alter table t_frag_code add column access_cnt INTEGER NOT NULL DEFAULT 0",
        (), // empty list of parameters.
      )?;
    }
    Ok(())
}

fn selectbyname(conn: &Connection, name:&str)->rusqlite::Result<Vec<FragCode>>{
  let mut stmt = conn.prepare("SELECT id, abbr, code FROM t_frag_code where abbr like ?||'%' order by abbr asc, access_cnt desc")?;
  let data_iter = stmt.query_map([name], |row| {
      Ok(FragCode {
          id: row.get(0)?,
          abbr: row.get(1)?,
          code: row.get(2)?,
      })
    })?;
  
  let mut nv = Vec::new();
  for item in data_iter {
    nv.push(item.unwrap());
  }
  Ok(nv)
}

fn selectall(conn: &Connection)->rusqlite::Result<Vec<FragCode>>{
  let mut stmt = conn.prepare("SELECT id, abbr, code FROM t_frag_code order by access_cnt desc,id desc")?;
  let data_iter = stmt.query_map([], |row| {
      Ok(FragCode {
          id: row.get(0)?,
          abbr: row.get(1)?,
          code: row.get(2)?,
      })
    })?;
  
  let mut nv = Vec::new();
  for item in data_iter {
    nv.push(item.unwrap());
  }
  Ok(nv)
}

#[tauri::command]
fn list(name: &str) ->String{
  let conn = connect();
  let v = listjson(&conn, name).unwrap();
  conn.close();
  v
}

fn listjson(conn: &Connection, name: &str) -> Result<String> {
  let v = if name.eq(""){
    selectall(&conn).unwrap()
  }else{
    selectbyname(&conn, name).unwrap()
  };
  let j = serde_json::to_string(&v)?;
  Ok(j)
}


#[tauri::command]
fn add(abbr: &str, code: &str){
  let conn = connect();
  let fc = FragCode {
    id: 0,
    abbr: abbr.to_string(),
    code: code.to_string(),
  };
  let v = vec![fc];
  insert(&conn, &v).unwrap();
  conn.close();
}


#[tauri::command]
fn remove(id: i32){
  let conn = connect();
  delete(&conn, id).unwrap();
  conn.close();
}

#[tauri::command]
fn access(id: i32){
  let conn = connect();
  conn.execute(
    "update t_frag_code set access_cnt = access_cnt + 1 where id = ?1",
    [id],
  ).unwrap();
  conn.close();
}

#[tauri::command]
fn toggle(app: AppHandle){
  let window = app.get_window("main").unwrap();
  if window.is_visible().unwrap(){
    window.hide().unwrap();
  }else{
    window.show().unwrap();
  }
}


fn handler(app: &AppHandle, event: SystemTrayEvent) {
  // 获取应用窗口
  let window = app.get_window("main").unwrap();
  let parent_window = Some(&window);
  // 匹配点击事件
  match event {
      // 左键点击
      SystemTrayEvent::LeftClick {
          position: _,
          size: _,
          ..
      } => {
          println!("system tray received a left click");
      }
      // 右键点击
      SystemTrayEvent::RightClick {
          position: _,
          size: _,
          ..
      } => {
          println!("system tray received a right click");
      }
      // 双击，macOS / Linux 不支持
      SystemTrayEvent::DoubleClick {
          position: _,
          size: _,
          ..
      } => {
          println!("system tray received a double click");
      }
      // 根据菜单 id 进行事件匹配
      SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
          "quit" => {
            std::process::exit(0);
          }
          "show" => {
            window.set_always_on_top(true).unwrap();
            window.show().unwrap();
          }
          "hide" => {
            window.hide().unwrap();
          }
          _ => {}
      },
      _ => {}
  }
}

fn main() {
  let conn = connect();
  createtable(&conn).unwrap();
  conn.close();


  let traymenu = SystemTrayMenu::new()
  .add_item(CustomMenuItem::new("hide".to_string(), "Hide"))
  .add_item(CustomMenuItem::new("show".to_string(), "Show"))
  .add_native_item(SystemTrayMenuItem::Separator)
  .add_item(CustomMenuItem::new("quit".to_string(), "Quit"));

  tauri::Builder::default()
    .system_tray(SystemTray::new().with_menu(traymenu))
    .on_system_tray_event(handler)
    .invoke_handler(tauri::generate_handler![list, add, remove, toggle, access])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
