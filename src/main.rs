#[macro_use]
extern crate tracing;

mod consts;
mod structs;
mod mqtt_actor;
mod mqtt_handler;

use core::time::Duration;
use std::collections::HashMap;
use std::fs;
use tokio::net::TcpStream;
use tokio::time::sleep;
use tokio_modbus::prelude::*;
use tokio_modbus::{Address, Quantity};
use tokio_serial::SerialStream;
use crate::structs::GrowattModel;
use console_subscriber as tokio_console_subscriber;
use serde_json::Value;
use tracing_subscriber::{EnvFilter, Registry, prelude::*};
use tracing_subscriber::fmt::format::FmtSpan;
use crate::mqtt_handler::MqttActorHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenv::dotenv();

    //region console logging
    let console_layer = tokio_console_subscriber::spawn();
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("warn"))
        .unwrap();
    let format_layer = tracing_subscriber::fmt::layer()
        .event_format(
            tracing_subscriber::fmt::format()
                .with_file(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_line_number(true),
        )
        .with_span_events(FmtSpan::NONE);


    let subscriber = Registry::default()
        .with(console_layer)
        .with(filter_layer)
        .with(format_layer);
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");
    //endregion

    let mqtt = MqttActorHandler::new();


    let slave = Slave(0x01);

    //let tty_path = "/dev/ttyUSB1";
    //let builder = tokio_serial::new(tty_path, 9600);
    //let port = SerialStream::open(&builder).unwrap();
    let mut port = TcpStream::connect("10.174.5.52:503").await?;

    let mut ctx = rtu::attach_slave(port, slave);

    let mut pointsmap: HashMap<Address, i16> = HashMap::new();

    let search: bool = false;
    let points = read_json_file("models/spf3000tl.json")?;

    let mut skip = points.get_points().iter().map(|p| p.address).collect::<Vec<Address>>();

    // bouncers
    skip.append(&mut vec![0xbc, 0xd8, 0x1b3, 0xea]);

    while search {
        println!("--------------------------------------------------");
        for addr in 0..500 {
            if skip.contains(&addr) {
                continue;
            }
            let previous = pointsmap.get(&addr);
            if previous.is_some() {
                if *previous.unwrap() == 0 {
                    continue;
                }
            }
            let rsp = ctx.read_input_registers(addr, 1).await??;
            sleep(Duration::from_millis(500)).await;
            let v = rsp[0] as i16;
            if v > 0 {
                match previous {
                    Some(p) => {
                        if *p == v {
                            continue;
                        }
                        println!("(0x{addr:02x}): {p} -> {v}");
                    }
                    None => {
                        println!("(0x{addr:02x}): {v}");
                    }
                }
            }
            pointsmap.insert(addr, v);
        }
    }


    loop {
        println!("-------------------");
        for p in points.get_points() {
            let rsp = ctx.read_input_registers(p.address, p.length as Quantity).await??;
            for v in rsp.iter() {
                let v2 = *v as i16;
                let sf_v = (v2 as f64) * 10.0_f64.powi(p.scale_factor.into());
                println!("{}(0x{:02x}): {sf_v:.2}{}", p.name, p.address, p.uom);

                let topic: String = format!("growatt/{}", p.name);
                let mut data: HashMap<String, Value> = HashMap::new();
                data.insert("name".to_string(), p.name.clone().into());
                data.insert("value".to_string(), sf_v.into());
                data.insert("uom".to_string(), p.uom.clone().into());
                let payload: String = serde_json::to_string(&data).unwrap();
                if let Err(e) = mqtt.publish(topic, payload).await {
                    panic!("Fatal error with sending payload via mqtt: {e}");
                }

            }
            sleep(Duration::from_millis(850)).await;
        }
    }

    println!("Disconnecting");
    ctx.disconnect().await?;

    Ok(())
}

fn string_proc(data: &Vec<u16>) -> String {
    let bytes: Vec<u8> = data.iter().fold(vec![], |mut x, elem| {
        let f = elem.to_be_bytes();
        x.append(&mut f.to_vec());
        x
    });
    match String::from_utf8(bytes) {
        Ok(s) => s,
        Err(e) => format!("ERROR: {e}"),
    }
}
fn read_json_file(path: &str) -> Result<GrowattModel, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path)?;
    let data: GrowattModel = serde_json::from_str(&contents)?;
    Ok(data)
}
