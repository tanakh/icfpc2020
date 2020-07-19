use crate::game::*;
use crate::value::*;
use anyhow::{bail, Result};
use std::io::{self, Write};

const JOIN_REQUEST_TAG: i128 = 2;
const START_REQUEST_TAG: i128 = 3;
const COMMAND_REQUEST_TAG: i128 = 4;

pub fn send_join_request() -> Result<Response> {
    let player_key: i128 = std::env::args().nth(1).unwrap().parse().unwrap();
    send_and_receive_game_state(&Value::Cons(
        Box::new(Value::Int(JOIN_REQUEST_TAG)),
        Box::new(Value::Cons(
            Box::new(Value::Int(player_key)),
            Box::new(Value::Cons(Box::new(Value::Nil), Box::new(Value::Nil))),
        )),
    ))
}

pub fn send_start_request(params: &Param) -> Result<Response> {
    let player_key: i128 = std::env::args().nth(1).unwrap().parse().unwrap();
    let is_tutorial: bool = std::env::vars().any(|(key, _)| key == "TUTORIAL_MODE");
    let params = if is_tutorial {
        Value::Nil
    } else {
        Value::Cons(
            Box::new(Value::Int(params.energy as i128)),
            Box::new(Value::Cons(
                Box::new(Value::Int(params.laser_power as i128)),
                Box::new(Value::Cons(
                    Box::new(Value::Int(params.cool_down_per_turn as i128)),
                    Box::new(Value::Cons(
                        Box::new(Value::Int(params.life as i128)),
                        Box::new(Value::Nil),
                    )),
                )),
            )),
        )
    };
    send_and_receive_game_state(&Value::Cons(
        Box::new(Value::Int(START_REQUEST_TAG)),
        Box::new(Value::Cons(
            Box::new(Value::Int(player_key)),
            Box::new(Value::Cons(Box::new(params), Box::new(Value::Nil))),
        )),
    ))
}

pub fn send_command_request(it: &mut impl Iterator<Item = Command>) -> Result<Response> {
    let commands = it.fold(Value::Nil, |acc, x| {
        Value::Cons(Box::new(x.to_value()), Box::new(acc))
    });
    let player_key: i128 = std::env::args().nth(1).unwrap().parse().unwrap();
    send_and_receive_game_state(&Value::Cons(
        Box::new(Value::Int(COMMAND_REQUEST_TAG)),
        Box::new(Value::Cons(
            Box::new(Value::Int(player_key)),
            Box::new(Value::Cons(Box::new(commands), Box::new(Value::Nil))),
        )),
    ))
}

fn parse_current_game_state(val: &Value) -> CurrentGameState {
    match *val {
        Value::Int(0) => CurrentGameState::START,
        Value::Int(1) => CurrentGameState::PLAYING,
        Value::Int(2) => CurrentGameState::END,
        _ => panic!(),
    }
}

fn parse_obstacle(val: Value) -> Result<Option<Obstacle>> {
    Ok(if let Some(val) = to_option(val) {
        Some(match to_vec(val.clone())?.as_slice() {
            [gravity_radius, stage_half_size] => Obstacle {
                gravity_radius: to_int(gravity_radius)? as usize,
                stage_half_size: to_int(stage_half_size)? as usize,
            },
            _ => bail!("unexpected value: {}", val.to_string()),
        })
    } else {
        None
    })
}

fn parse_stage_data(val: Value) -> Result<StageData> {
    Ok(match to_vec(val.clone())?.as_slice() {
        [total_turns, role, _2, obstacle, _3] => match to_vec(_2.clone())?.as_slice() {
            [_20, _21, _22] => StageData {
                total_turns: to_int(total_turns)? as usize,
                self_role: parse_role(role.clone())?,
                _2: (
                    to_int(_20)? as isize,
                    to_int(_21)? as isize,
                    to_int(_22)? as isize,
                ),
                obstacle: parse_obstacle(obstacle.clone())?,
                _3: to_vec(_3.clone())?
                    .into_iter()
                    .map(|val| to_int(&val).map(|r| r as isize))
                    .collect::<Result<Vec<_>>>()?,
            },
            _ => bail!("unexpected value: {}", _2.to_string()),
        },
        _ => bail!("unexpected value: {}", val.to_string()),
    })
}

fn parse_position(val: Value) -> Result<(isize, isize)> {
    Ok(match val {
        Value::Cons(x, y) => (to_int(&*x)? as isize, to_int(&*y)? as isize),
        _ => bail!("Unexpected value: ".to_string() + &val.to_string()),
    })
}

fn parse_point(val: Value) -> Result<Point> {
    Ok(match val {
        Value::Cons(x, y) => Point {
            x: to_int(&*x)? as isize,
            y: to_int(&*y)? as isize,
        },
        _ => bail!("Unexpected value: ".to_string() + &val.to_string()),
    })
}

fn parse_role(val: Value) -> Result<Role> {
    Ok(match to_int(&val)? {
        0 => Role::ATTACKER,
        1 => Role::DEFENDER,
        _ => bail!("unexpected value: {}", val.to_string()),
    })
}

fn parse_params(val: Value) -> Result<Param> {
    Ok(match to_vec(val.clone())?.as_slice() {
        [energy, laser_power, cool_down_per_turn, life] => Param {
            energy: to_int(energy)? as usize,
            laser_power: to_int(laser_power)? as usize,
            cool_down_per_turn: to_int(cool_down_per_turn)? as usize,
            life: to_int(life)? as usize,
        },
        _ => bail!("unexpected value: {}", val.to_string()),
    })
}

fn parse_machine(val: Value) -> Result<Machine> {
    Ok(match to_vec(val.clone())?.as_slice() {
        [role, machine_id, position, velocity, params, heat, _1, _2] => Machine {
            role: parse_role(role.clone())?,
            machine_id: to_int(machine_id)? as isize,
            position: parse_point(position.clone())?,
            velocity: parse_point(velocity.clone())?,
            params: parse_params(params.clone())?,
            heat: to_int(heat)? as usize,
            _1: to_int(_1)? as isize,
            _2: to_int(_2)? as isize,
            generated_heat: 0,
            attack_heat: 0,
        },
        _ => bail!("unexpected value: ".to_string() + &val.to_string()),
    })
}

fn parse_action_result(val: Value) -> Result<Option<ActionResult>> {
    let vals = to_vec(val.clone())?;
    if vals.is_empty() {
        Ok(None)
    } else {
        let action_result = match (to_int(&vals[0].clone())?, vals.as_slice()) {
            (0, [_, a]) => ActionResult::Thruster {
                a: parse_point(a.clone())?,
            },
            (1, [_, power, area]) => ActionResult::Bomb {
                power: to_int(power)? as usize,
                area: to_int(area)? as usize,
            },
            (2, [_, opponent]) => ActionResult::Laser {
                opponent: parse_point(opponent.clone())?,
            },
            (3, [_, params]) => ActionResult::Split {
                params: parse_params(params.clone())?,
            },
            _ => panic!("unexpected value: ".to_string() + &val.to_string()),
        };
        Ok(Some(action_result))
    }
}

fn parse_machine_and_action_result(val: Value) -> Result<(Machine, Option<ActionResult>)> {
    match to_vec(val.clone())?.as_slice() {
        [machine, action_result] => Ok((
            parse_machine(machine.clone())?,
            parse_action_result(action_result.clone())?,
        )),
        _ => bail!("unexpected value: {}", val.to_string()),
    }
}

fn parse_current_state(val: Value) -> Result<Option<CurrentState>> {
    match to_vec(val.clone())?.as_slice() {
        [turn, obstacle, machines] => Ok(Some(CurrentState {
            turn: to_int(turn)? as usize,
            obstacle: parse_obstacle(obstacle.clone())?,
            machines: to_vec(machines.clone())?
                .into_iter()
                .map(|val| parse_machine_and_action_result(val))
                .collect::<Result<Vec<_>>>()?,
        })),
        [] => Ok(None),
        _ => bail!("unexpected value: {}", val.to_string()),
    }
}

fn parse_response(val: Value) -> Result<Response> {
    match to_vec(val.clone())?.as_slice() {
        [tag, current_game_state, stage_data, current_state] => {
            if to_int(tag)? != 1 {
                bail!("tag is not 1: {}", tag.to_string());
            }
            Ok(Response {
                current_game_state: parse_current_game_state(current_game_state),
                stage_data: parse_stage_data(stage_data.clone())?,
                current_state: parse_current_state(current_state.clone())?,
            })
        }
        [tag] if to_int(tag)? == 0 => bail!("wrong request"),
        _ => bail!("unexpected response value: {}", val.to_string()),
    }
}

fn send_and_receive_game_state(val: &Value) -> Result<Response> {
    println!("{}", modulate_to_string(&val));
    io::stdout().flush();
    eprintln!("send: {}", val.to_string());
    let mut resp = String::new();
    io::stdin().read_line(&mut resp).unwrap();
    let resp = demodulate_from_string(&resp).unwrap();
    let resp = parse_response(resp);
    eprintln!("recieve: {:?}", resp);
    resp
}
