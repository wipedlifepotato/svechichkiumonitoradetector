use mlua::{Lua, Result, Value};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::stream::PricePair;
use tokio::sync::mpsc;
use std::path::Path;
use std::fs;
use crate::ArbitrageStrategy;

#[macro_export]
macro_rules! global_lua_env_init {
    ( $lua_global:expr, $expr:expr, $def: expr ) => {
        $lua_global.set($expr, std::env::var($expr).unwrap_or($def.to_string()))?
    }
}

pub fn load_scripts(lua: &Lua, scripts_dir: &str) -> Result<()> {
    let path = Path::new(scripts_dir);
    
    if !path.exists() {
        fs::create_dir_all(path).ok();
        println!("Created scripts directory: {}", scripts_dir);
        return Ok(());
    }

    for entry in fs::read_dir(path).map_err(|e| mlua::Error::RuntimeError(e.to_string()))? {
        let entry = entry.map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
        let file_path = entry.path();

        if file_path.is_file() && file_path.extension().and_then(|s| s.to_str()) == Some("lua") {
            let script_content = fs::read_to_string(&file_path)
                .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
            
            println!("Loading script: {:?}", file_path.file_name().unwrap());
            
            lua.load(&script_content).exec()?;
        }
    }
    Ok(())
}

pub fn init_lua(
    app_data: Arc<Mutex<HashMap<String, Vec<PricePair>>>>,
    pairs: Vec<String>,
    strategy: Arc<Box<dyn ArbitrageStrategy>> 
) -> mlua::Result<Lua> {
    let lua = Lua::new();
    let globals = lua.globals();

    let s_ptr = Arc::clone(&strategy);
    let send_alert = lua.create_function(move |_, msg: String| {
        let s = Arc::clone(&s_ptr);
        tokio::spawn(async move {
            s.alert(&msg).await;
        });
        Ok(())
    })?;

    globals.set("send_to_rust_alert", send_alert)?;	
	let pairs_table = lua.create_table()?;
    
    let cfg = crate::config::Config::load();
    
    for (i, pair) in cfg.pairs.into_iter().enumerate() {
        pairs_table.set(i + 1, pair)?;
    }
    
    globals.set("active_pairs", pairs_table)?;
	let lua_sleep = lua.create_function(|_, ms: u64| {
		std::thread::sleep(std::time::Duration::from_millis(ms));
		Ok(())
	})?;
	lua.globals().set("sleep", lua_sleep)?;

    let _ = global_lua_env_init!(globals, "BOT_TG_FATHER_KEY", "");
    let _ = global_lua_env_init!(globals, "BOT_TG_USERID", "");
    let _ = global_lua_env_init!(globals, "ALERT_DESKTOP_ENABLED", "false");
    let _ = global_lua_env_init!(globals, "LERT_TELEGRAM_ENABLED", "false");
    let _ = global_lua_env_init!(globals, "CHARTS_URL", "http://127.0.0.1:3000/charts");
    let _ = global_lua_env_init!(globals, "GEMINI_API_KEY", "");
    let _ = global_lua_env_init!(globals, "PROXY_URL", "");
    let _ = global_lua_env_init!(globals, "LANGUAGE_AI", "ENGLISH");

    let data_for_lua = Arc::clone(&app_data);
    let get_market_data = lua.create_function(move |lua, pair: String| {
        let data = data_for_lua.lock().map_err(|_| {
            mlua::Error::RuntimeError("Locking mutex failed".to_string())
        })?;

        if let Some(history) = data.get(&pair.to_uppercase()) {
            let table = lua.create_table()?; 
            for (i, price_point) in history.iter().enumerate() {
                let entry = lua.create_table()?;
                entry.set("bid_price", price_point.bid_price.clone())?;
                entry.set("ask_price", price_point.ask_price.clone())?;
                entry.set("timestamp", price_point.timestamp)?;
                table.set(i + 1, entry)?;
            }
            Ok(Value::Table(table))
        } else {
            Ok(Value::Nil)
        }
    })?;

    globals.set("get_market_data", get_market_data)?;

    Ok(lua)    
}
