use anyhow::{anyhow, bail, Context, Result};
use rust_game_base::*;
use std::cmp::{max, min};

// 知見
// レーザーの威力はmin(初期設定のlaser_power, 発射時のパラメーター)
// heatは実際に発射された威力分上昇

const MAX_STEP: usize = 256;

struct Bot {
    stage: CurrentGameState,
    static_info: StageData,
    state: CurrentState,
}

static VECT: &[Point] = &[
    Point::new(-1, -1),
    Point::new(-1, 0),
    Point::new(-1, 1),
    Point::new(0, -1),
    Point::new(0, 1),
    Point::new(1, -1),
    Point::new(1, 0),
    Point::new(1, 1),
    Point::new(0, 0),
];

impl Bot {
    fn new() -> Result<Bot> {
        let resp = send_join_request()?;
        Ok(Bot {
            stage: resp.current_game_state,
            static_info: resp.stage_data,
            state: Default::default(),
        })
    }

    fn apply_response(&mut self, resp: Response) {
        self.stage = resp.current_game_state;
        assert_eq!(self.static_info, resp.stage_data);
        self.state = resp.current_state.unwrap();
    }

    fn start(&mut self) -> Result<()> {
        assert_eq!(self.stage, CurrentGameState::START);
        dbg!(&self.static_info);

        // FIXME: ???
        let mut param_rest = self.static_info.initialize_param.total_cost;

        let mut param = Param {
            energy: 0,
            laser_power: 0,
            life: 0,
            cool_down_per_turn: 0,
        };

        // パラメーター割り振り
        // attackerの時はレーザーに多めに、
        // defenderの時はレーザーに割り振らないようにするといい？

        while param_rest > 0 {
            if param.life == 0 {
                param.life += 1;
                param_rest -= 2;
                continue;
            }

            if param_rest >= 12
                && param.cool_down_per_turn * 12 <= param.energy
                && param.cool_down_per_turn * 12 <= param.laser_power * 4
            // && param.cool_down_per_turn * 12 <= param.life * 2
            {
                param.cool_down_per_turn += 1;
                param_rest -= 12;
                continue;
            }

            if param_rest >= 4
                && param.laser_power * 4 <= param.energy
                // && param.laser_power * 4 <= param.life * 2
                && param.laser_power * 4 <= param.cool_down_per_turn * 12
            {
                param.laser_power += 1;
                param_rest -= 4;
                continue;
            }

            // if param_rest >= 2
            //     && param.life * 2 <= param.energy
            //     && param.life * 2 <= param.laser_power * 4
            //     && param.life * 2 <= param.cool_down_per_turn * 12
            // {
            //     param.life += 1;
            //     param_rest -= 2;
            //     continue;
            // }

            param.energy += 1;
            param_rest -= 1;
        }

        dbg!(&param);

        assert!(
            param.energy + param.laser_power * 4 + param.life * 2 + param.cool_down_per_turn * 12
                <= self.static_info.initialize_param.total_cost as usize
        );

        self.apply_response(send_start_request(&param)?);
        Ok(())
    }

    fn step(&mut self) -> Result<()> {
        assert_eq!(self.stage, CurrentGameState::PLAYING);
        let cmds = self.think();
        self.apply_response(send_command_request(&mut cmds.into_iter())?);
        Ok(())
    }

    fn think(&mut self) -> Vec<Command> {
        dbg!(self.static_info.self_role);
        dbg!(&self.state);

        let mut cmds = vec![];

        // * 楕円軌道に乗る
        // * 位置関係がいい感じならビームを打つ
        // * 分裂をどうするか

        for m in self.state.machines.iter() {
            let m = m.0;
            if m.role != self.static_info.self_role {
                continue;
            }

            // 一番長く生き残るところに行く
            let (best_time, _, best_v) = VECT
                .iter()
                .map(|v| (self.live_time(&m, v), -(v.x.abs() + v.y.abs()), v))
                .max_by_key(|r| r.0)
                .unwrap();

            if best_v.x == 0 && best_v.y == 0 {
                continue;
            }

            dbg!(best_time, best_v);

            cmds.push(Command::Thrust(m.machine_id as _, *best_v));
        }

        cmds
    }

    fn live_time(&self, r: &Machine, acc: &Point) -> usize {
        let step_limit = MAX_STEP - self.state.turn;

        let mut ret = 0;
        let mut v = r.velocity - *acc;
        let mut cur = r.position;

        while ret < step_limit {
            v += get_gravity(&self.state, &cur);
            cur += v;

            if !self.is_safe(&cur) {
                break;
            }

            ret += 1;
        }

        dbg!(ret);

        ret
    }

    fn is_safe(&self, p: &Point) -> bool {
        if let Some(obs) = &self.static_info.obstacle {
            if p.x.abs() <= obs.gravity_radius as isize && p.y.abs() <= obs.gravity_radius as isize
            {
                return false;
            }

            if p.x.abs() > obs.stage_half_size as isize || p.y.abs() > obs.stage_half_size as isize
            {
                return false;
            }
        }
        true
    }
}

// Returns the gravity.
pub fn get_gravity(state: &CurrentState, pos: &Point) -> Point {
    if state.obstacle.is_none() {
        return Point { x: 0, y: 0 };
    }

    Point {
        x: if pos.x.abs() < pos.y.abs() {
            0
        } else {
            -pos.x.signum()
        },
        y: if pos.y.abs() < pos.x.abs() {
            0
        } else {
            -pos.y.signum()
        },
    }
}

pub fn clamp<T: Ord>(input: T, min_v: T, max_v: T) -> T {
    max(min_v, min(max_v, input))
}

fn main() -> Result<()> {
    let mut bot = Bot::new()?;
    bot.start()?;
    while bot.stage != CurrentGameState::END {
        bot.step()?;
    }
    Ok(())
}
