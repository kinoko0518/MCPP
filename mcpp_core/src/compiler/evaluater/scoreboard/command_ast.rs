use crate::evaluater::Types;

use super::{comparison_operation, Scoreboard};
use crate::compiler::evaluater::Operator;


use super::{NAMESPACE, FLOAT_MAGNIFICATION};

#[derive(Debug, Clone)]
pub enum ScoreAST {
    CalcScore(CalcScore),
    AddRemNum(AddRemNum),
    AssignScore(AssignScore),
    AssignNum(AssignNum),
    BoolifyCondition(ExecuteConstructer),
    Free(Free)
}
impl Serialise for ScoreAST {
    fn serialise(&self) -> String {
        match self {
            ScoreAST::AddRemNum(a) => a.serialise(),
            ScoreAST::AssignNum(a) => a.serialise(),
            ScoreAST::AssignScore(a) => a.serialise(),
            ScoreAST::CalcScore(c) => c.serialise(),
            ScoreAST::BoolifyCondition(b) => b.serialise(),
            ScoreAST::Free(f) => f.serialise()
        }
    }
}

pub struct FormulaConstructer {
    commands: Vec<ScoreAST>,
    temp_scores: Vec<Scoreboard>
}

fn get_const(constant:i32) -> Scoreboard {
    Scoreboard {
        name: constant.to_string(),
        scope: vec!["CONSTANT".to_string()],
        datatype: Types::Int
    }
}

impl FormulaConstructer {
    pub fn new() -> Self {
        FormulaConstructer {
            commands: Vec::new(),
            temp_scores: Vec::new()
        }
    }
    pub fn calc_score(&mut self, left:&Scoreboard, operator:String, right:&Scoreboard) -> &mut Self {
        self.commands.push(ScoreAST::CalcScore(
            CalcScore {
                left: ScoreTarget::from(left),
                operator: operator,
                right: ScoreTarget::from(right)
            }
        ));
        return self
    }
    pub fn calc_num(&mut self, left:&Scoreboard, operator:String, right:i32) -> &mut Self {
        let constant = get_const(right);
        self.commands.push(ScoreAST::AssignNum(
            AssignNum { left: ScoreTarget::from(&constant), right: right }
        ));
        self.temp_scores.push(constant);
        let constant = self.temp_scores.last().unwrap();
        self.commands.push(ScoreAST::CalcScore(
            CalcScore {
                left: ScoreTarget::from(left),
                operator: operator,
                right: ScoreTarget::from(constant)
            }
        ));
        return self
    }
    pub fn add_rem_num(&mut self, left:&Scoreboard, add_rem:String, right:i32) -> &mut Self {
        self.commands.push(ScoreAST::AddRemNum(
            AddRemNum { left: ScoreTarget::from(left), add_rem: add_rem, right: right }
        ));
        self
    }
    pub fn assign_score(&mut self, left:&Scoreboard, right:&Scoreboard) -> &mut Self {
        self.commands.push(ScoreAST::AssignScore(
            AssignScore { left: ScoreTarget::from(left), right: ScoreTarget::from(right) }
        ));
        self
    }
    pub fn assign_num(&mut self, left:&Scoreboard, right:i32) -> &mut Self {
        self.commands.push(ScoreAST::AssignNum(
            AssignNum { left: ScoreTarget::from(left), right: right }
        ));
        self
    }
    pub fn intify(&mut self, target:&Scoreboard) -> &mut Self {
        self.calc_num(target, "/=".to_string(), FLOAT_MAGNIFICATION)
    }
    pub fn fltify(&mut self, target:&Scoreboard) -> &mut Self {
        self.calc_num(target, "*=".to_string(), FLOAT_MAGNIFICATION)
    }
    pub fn boolify_score_comparison(&mut self, left:&Scoreboard, comparison:String, right:&Scoreboard) -> &mut Self {
        self.commands.push(ScoreAST::BoolifyCondition(
            ExecuteConstructer {
                conditions: vec![ConditionAST::Comparison(Comparison {
                    is_unless: false,
                    left: ScoreTarget::from(left),
                    comparison: comparison,
                    right: ScoreTarget::from(right)
                })]
            }
        ));
        self
    }
    pub fn boolify_num_comparison(&mut self, left:&Scoreboard, comparison:String, right:i32) -> &mut Self {
        let constant = get_const(right);
        let const_target = ScoreTarget::from(&constant);
        self.commands.push(ScoreAST::AssignNum(
            AssignNum { left: const_target.clone(), right: right }
        ));
        self.commands.push(ScoreAST::BoolifyCondition(
            ExecuteConstructer {
                conditions: vec![ConditionAST::Comparison(Comparison {
                    is_unless: false,
                    left: ScoreTarget::from(left),
                    comparison: comparison,
                    right: const_target
                })]
            }
        ));
        self.temp_scores.push(constant);
        self
    }
    pub fn validate_bool(&mut self, target:&Scoreboard) -> &mut Self {
        let constant_0 = get_const(0);
        self.commands.push(ScoreAST::AssignNum(
            AssignNum { left: ScoreTarget::from(&constant_0), right: 0 }
        ));
        self.temp_scores.push(constant_0);
        self.commands.push(ScoreAST::BoolifyCondition(
            ExecuteConstructer {
                conditions: vec![ConditionAST::Comparison(Comparison {
                    is_unless: true,
                    left: ScoreTarget::from(target),
                    comparison: "==".to_string(),
                    right: ScoreTarget::from(self.temp_scores.last().unwrap())
                })]
            }
        ));
        self
    }
    pub fn free(&mut self, target:&Scoreboard) -> &mut Self {
        self.commands.push(ScoreAST::Free(
            Free { target: ScoreTarget::from(target) }
        ));
        self
    }
    pub fn build(&mut self) -> Vec<ScoreAST> {
        for tmp in &self.temp_scores {
            self.commands.push(ScoreAST::Free(
                Free { target: ScoreTarget::from(tmp) }
            ));
        }
        self.commands.clone()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScoreTarget {
    pub player: String,
    pub objective: String,
}

impl From<&Scoreboard> for ScoreTarget {
    fn from(sb: &Scoreboard) -> Self {
        ScoreTarget {
            player: sb.get_mcname(),
            objective: NAMESPACE.to_string()
        }
    }
}

pub trait Serialise {
    fn serialise(&self) -> String;
}

#[derive(Debug, Clone)]
pub struct CalcScore {
    left: ScoreTarget,
    operator: String,
    right: ScoreTarget
}
impl Serialise for CalcScore {
    fn serialise(&self) -> String {
        format!(
            "scoreboard players operation {} {} {} {} {}",
            self.left.player,
            self.left.objective,
            self.operator,
            self.right.player,
            self.right.objective
        )
    }
}

#[derive(Debug, Clone)]
pub struct AddRemNum {
    left: ScoreTarget,
    add_rem: String,
    right: i32
}
impl Serialise for AddRemNum {
    fn serialise(&self) -> String {
        format!(
            "scoreboard players {} {} {} {}",
            self.add_rem,
            self.left.player,
            NAMESPACE,
            self.right
        )
    }
}

#[derive(Debug, Clone)]
pub struct AssignScore {
    left: ScoreTarget,
    right: ScoreTarget
}
impl Serialise for AssignScore {
    fn serialise(&self) -> String {
        format!(
            "scoreboard players operation {} {} = {} {}",
            self.left.player,
            self.left.objective,
            self.right.player,
            self.right.objective
        )
    }
}

#[derive(Debug, Clone)]
pub struct AssignNum {
    left: ScoreTarget,
    right: i32
}
impl Serialise for AssignNum {
    fn serialise(&self) -> String {
        format!("scoreboard players set {} {} {}", self.left.player, NAMESPACE, self.right)
    }
}

#[derive(Debug, Clone)]
pub struct BoolifyCondition {
    contain_to: ScoreTarget,
    execute: ExecuteConstructer
}
impl Serialise for BoolifyCondition {
    fn serialise(&self) -> String {
        let temp = super::get_calc_temp(Types::Bool);
        let temp_target = ScoreTarget::from(&temp);
        [
            AssignNum { left:temp_target.clone(), right: 0 }.serialise(),
            format!(
                "{}{}",
                self.execute.serialise(),
                AssignNum { left: temp_target.clone(), right: 1 }.serialise()
            ),
            AssignScore {
                left: self.contain_to.clone(), right: temp_target.clone()
            }.serialise(),
            Free { target: temp_target }.serialise()
        ].join("\n")
    }
}

#[derive(Debug, Clone)]
pub struct Free {
    target: ScoreTarget
}
impl Serialise for Free {
    fn serialise(&self) -> String {
        format!("scoreboard players reset {} {}", self.target.player, self.target.objective)
    }
}


#[derive(Debug, Clone)]
pub struct ExecuteConstructer {
    conditions: Vec<ConditionAST>
}
impl Serialise for ExecuteConstructer {
    fn serialise(&self) -> String {
        format!(
            "execute {} run ",
            self.conditions
                .iter()
                .map(|c| c.serialise())
                .collect::<Vec<String>>()
                .join(" ")
        )
    }
}
#[derive(Debug, Clone)]
enum ConditionAST {
    Comparison(Comparison)
}
impl Serialise for ConditionAST {
    fn serialise(&self) -> String {
        match self {
            ConditionAST::Comparison(c) => c.serialise()
        }
    }
}

#[derive(Debug, Clone)]
struct Comparison {
    is_unless: bool,
    left: ScoreTarget,
    comparison: String,
    right: ScoreTarget
}
impl Serialise for Comparison {
    fn serialise(&self) -> String {
        let neq = comparison_operation::Comparison::Neq.to_str();
        let eq = comparison_operation::Comparison::Eq.to_str();
        if self.comparison.as_str() == neq {
            format!(
                "unless score {} {} {} {} {}",
                self.left.player,
                self.left.objective,
                eq,
                self.right.player,
                self.right.objective
            )
        } else {
            format!(
                "{} score {} {} {} {} {}",
                if self.is_unless {
                    "unless"
                } else {
                    "if"
                },
                self.left.player,
                self.left.objective,
                self.comparison,
                self.right.player,
                self.right.objective
            )
        }
    }
}