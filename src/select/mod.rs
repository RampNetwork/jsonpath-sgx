use std::collections::HashSet;
use std::fmt;
use std::string::{String, ToString};
use std::borrow::ToOwned;
use std::vec::Vec;

use array_tool::vec::{Intersect, Union};
use serde_json::{Number, Value};

use parser::parser::*;

fn to_f64(n: &Number) -> f64 {
    if n.is_i64() {
        n.as_i64().unwrap() as f64
    } else if n.is_f64() {
        n.as_f64().unwrap()
    } else {
        n.as_u64().unwrap() as f64
    }
}

trait Cmp {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool;

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool;

    fn cmp_string(&self, v1: &String, v2: &String) -> bool;

    fn cmp_json<'a>(&self, v1: &Vec<&'a Value>, v2: &Vec<&'a Value>) -> Vec<&'a Value>;

    fn default(&self) -> bool {
        false
    }
}

struct CmpEq;

impl Cmp for CmpEq {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool {
        v1 == v2
    }

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool {
        v1 == v2
    }

    fn cmp_string(&self, v1: &String, v2: &String) -> bool {
        v1 == v2
    }

    fn cmp_json<'a>(&self, v1: &Vec<&'a Value>, v2: &Vec<&'a Value>) -> Vec<&'a Value> {
        v1.intersect(v2.to_vec())
    }
}

struct CmpNe;

impl Cmp for CmpNe {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool {
        v1 != v2
    }

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool {
        v1 != v2
    }

    fn cmp_string(&self, v1: &String, v2: &String) -> bool {
        v1 != v2
    }

    fn cmp_json<'a>(&self, v1: &Vec<&'a Value>, v2: &Vec<&'a Value>) -> Vec<&'a Value> {
        v1.intersect_if(v2.to_vec(), |a, b| a != b)
    }
}

struct CmpGt;

impl Cmp for CmpGt {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool {
        v1 > v2
    }

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool {
        v1 > v2
    }

    fn cmp_string(&self, v1: &String, v2: &String) -> bool {
        v1 > v2
    }

    fn cmp_json<'a>(&self, _: &Vec<&'a Value>, _: &Vec<&'a Value>) -> Vec<&'a Value> {
        Vec::new()
    }
}

struct CmpGe;

impl Cmp for CmpGe {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool {
        v1 >= v2
    }

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool {
        v1 >= v2
    }

    fn cmp_string(&self, v1: &String, v2: &String) -> bool {
        v1 >= v2
    }

    fn cmp_json<'a>(&self, _: &Vec<&'a Value>, _: &Vec<&'a Value>) -> Vec<&'a Value> {
        Vec::new()
    }
}

struct CmpLt;

impl Cmp for CmpLt {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool {
        v1 < v2
    }

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool {
        v1 < v2
    }

    fn cmp_string(&self, v1: &String, v2: &String) -> bool {
        v1 < v2
    }

    fn cmp_json<'a>(&self, _: &Vec<&'a Value>, _: &Vec<&'a Value>) -> Vec<&'a Value> {
        Vec::new()
    }
}

struct CmpLe;

impl Cmp for CmpLe {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool {
        v1 <= v2
    }

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool {
        v1 <= v2
    }

    fn cmp_string(&self, v1: &String, v2: &String) -> bool {
        v1 <= v2
    }

    fn cmp_json<'a>(&self, _: &Vec<&'a Value>, _: &Vec<&'a Value>) -> Vec<&'a Value> {
        Vec::new()
    }
}

struct CmpAnd;

impl Cmp for CmpAnd {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool {
        *v1 && *v2
    }

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool {
        v1 > &0_f64 && v2 > &0_f64
    }

    fn cmp_string(&self, v1: &String, v2: &String) -> bool {
        !v1.is_empty() && !v2.is_empty()
    }

    fn cmp_json<'a>(&self, v1: &Vec<&'a Value>, v2: &Vec<&'a Value>) -> Vec<&'a Value> {
        v1.intersect(v2.to_vec())
    }
}

struct CmpOr;

impl Cmp for CmpOr {
    fn cmp_bool(&self, v1: &bool, v2: &bool) -> bool {
        *v1 || *v2
    }

    fn cmp_f64(&self, v1: &f64, v2: &f64) -> bool {
        v1 > &0_f64 || v2 > &0_f64
    }

    fn cmp_string(&self, v1: &String, v2: &String) -> bool {
        !v1.is_empty() || !v2.is_empty()
    }

    fn cmp_json<'a>(&self, v1: &Vec<&'a Value>, v2: &Vec<&'a Value>) -> Vec<&'a Value> {
        v1.union(v2.to_vec())
    }
}

#[derive(Debug)]
enum ExprTerm<'a> {
    String(String),
    Number(Number),
    Bool(bool),
    Json(Option<FilterKey>, Vec<&'a Value>),
}

impl<'a> ExprTerm<'a> {
    fn is_string(&self) -> bool {
        match &self {
            ExprTerm::String(_) => true,
            _ => false,
        }
    }

    fn is_number(&self) -> bool {
        match &self {
            ExprTerm::Number(_) => true,
            _ => false,
        }
    }

    fn is_bool(&self) -> bool {
        match &self {
            ExprTerm::Bool(_) => true,
            _ => false,
        }
    }

    fn is_json(&self) -> bool {
        match &self {
            ExprTerm::Json(_, _) => true,
            _ => false,
        }
    }

    fn cmp<C1: Cmp, C2: Cmp>(
        &self,
        other: &Self,
        cmp_fn: &C1,
        reverse_cmp_fn: &C2,
    ) -> ExprTerm<'a> {
        match &self {
            ExprTerm::String(s1) => match &other {
                ExprTerm::String(s2) => ExprTerm::Bool(cmp_fn.cmp_string(s1, s2)),
                ExprTerm::Json(_, _) => other.cmp(&self, reverse_cmp_fn, cmp_fn),
                _ => ExprTerm::Bool(cmp_fn.default()),
            },
            ExprTerm::Number(n1) => match &other {
                ExprTerm::Number(n2) => ExprTerm::Bool(cmp_fn.cmp_f64(&to_f64(n1), &to_f64(n2))),
                ExprTerm::Json(_, _) => other.cmp(&self, reverse_cmp_fn, cmp_fn),
                _ => ExprTerm::Bool(cmp_fn.default()),
            },
            ExprTerm::Bool(b1) => match &other {
                ExprTerm::Bool(b2) => ExprTerm::Bool(cmp_fn.cmp_bool(b1, b2)),
                ExprTerm::Json(_, _) => other.cmp(&self, reverse_cmp_fn, cmp_fn),
                _ => ExprTerm::Bool(cmp_fn.default()),
            },
            ExprTerm::Json(fk1, vec1) if other.is_string() => {
                let s2 = if let ExprTerm::String(s2) = &other {
                    s2
                } else {
                    unreachable!()
                };

                let ret: Vec<&Value> = vec1
                    .iter()
                    .filter(|v1| match v1 {
                        Value::String(s1) => cmp_fn.cmp_string(s1, s2),
                        Value::Object(map1) => {
                            if let Some(FilterKey::String(k)) = fk1 {
                                if let Some(Value::String(s1)) = map1.get(k) {
                                    return cmp_fn.cmp_string(s1, s2);
                                }
                            }
                            cmp_fn.default()
                        }
                        _ => cmp_fn.default(),
                    })
                    .map(|v| *v)
                    .collect();

                if ret.is_empty() {
                    ExprTerm::Bool(cmp_fn.default())
                } else {
                    ExprTerm::Json(None, ret)
                }
            }
            ExprTerm::Json(fk1, vec1) if other.is_number() => {
                let n2 = if let ExprTerm::Number(n2) = &other {
                    n2
                } else {
                    unreachable!()
                };
                let ret: Vec<&Value> = vec1
                    .iter()
                    .filter(|v1| match v1 {
                        Value::Number(n1) => cmp_fn.cmp_f64(&to_f64(n1), &to_f64(n2)),
                        Value::Object(map1) => {
                            if let Some(FilterKey::String(k)) = fk1 {
                                if let Some(Value::Number(n1)) = map1.get(k) {
                                    return cmp_fn.cmp_f64(&to_f64(n1), &to_f64(n2));
                                }
                            }
                            cmp_fn.default()
                        }
                        _ => cmp_fn.default(),
                    })
                    .map(|v| *v)
                    .collect();

                if ret.is_empty() {
                    ExprTerm::Bool(cmp_fn.default())
                } else {
                    ExprTerm::Json(None, ret)
                }
            }
            ExprTerm::Json(fk1, vec1) if other.is_bool() => {
                let b2 = if let ExprTerm::Bool(b2) = &other {
                    b2
                } else {
                    unreachable!()
                };
                let ret: Vec<&Value> = vec1
                    .iter()
                    .filter(|v1| match v1 {
                        Value::Bool(b1) => cmp_fn.cmp_bool(b1, b2),
                        Value::Object(map1) => {
                            if let Some(FilterKey::String(k)) = fk1 {
                                if let Some(Value::Bool(b1)) = map1.get(k) {
                                    return cmp_fn.cmp_bool(b1, b2);
                                }
                            }
                            cmp_fn.default()
                        }
                        _ => cmp_fn.default(),
                    })
                    .map(|v| *v)
                    .collect();

                if ret.is_empty() {
                    ExprTerm::Bool(cmp_fn.default())
                } else {
                    ExprTerm::Json(None, ret)
                }
            }
            ExprTerm::Json(_, vec1) if other.is_json() => match &other {
                ExprTerm::Json(_, vec2) => {
                    let vec = cmp_fn.cmp_json(vec1, vec2);
                    if vec.is_empty() {
                        ExprTerm::Bool(cmp_fn.default())
                    } else {
                        ExprTerm::Json(None, vec)
                    }
                }
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    fn eq(&self, other: &Self, ret: &mut Option<ExprTerm<'a>>) {
        debug!("eq - {:?} : {:?}", &self, &other);
        let _ = ret.take();
        let tmp = self.cmp(other, &CmpEq, &CmpEq);
        debug!("eq = {:?}", tmp);
        *ret = Some(tmp);
    }

    fn ne(&self, other: &Self, ret: &mut Option<ExprTerm<'a>>) {
        debug!("ne - {:?} : {:?}", &self, &other);
        let _ = ret.take();
        let tmp = self.cmp(other, &CmpNe, &CmpNe);
        debug!("ne = {:?}", tmp);
        *ret = Some(tmp);
    }

    fn gt(&self, other: &Self, ret: &mut Option<ExprTerm<'a>>) {
        debug!("gt - {:?} : {:?}", &self, &other);
        let _ = ret.take();
        let tmp = self.cmp(other, &CmpGt, &CmpLt);
        debug!("gt = {:?}", tmp);
        *ret = Some(tmp);
    }

    fn ge(&self, other: &Self, ret: &mut Option<ExprTerm<'a>>) {
        debug!("ge - {:?} : {:?}", &self, &other);
        let _ = ret.take();
        let tmp = self.cmp(other, &CmpGe, &CmpLe);
        debug!("ge = {:?}", tmp);
        *ret = Some(tmp);
    }

    fn lt(&self, other: &Self, ret: &mut Option<ExprTerm<'a>>) {
        debug!("lt - {:?} : {:?}", &self, &other);
        let _ = ret.take();
        let tmp = self.cmp(other, &CmpLt, &CmpGt);
        debug!("lt = {:?}", tmp);
        *ret = Some(tmp);
    }

    fn le(&self, other: &Self, ret: &mut Option<ExprTerm<'a>>) {
        debug!("le - {:?} : {:?}", &self, &other);
        let _ = ret.take();
        let tmp = self.cmp(other, &CmpLe, &CmpGe);
        debug!("le = {:?}", tmp);
        *ret = Some(tmp);
    }

    fn and(&self, other: &Self, ret: &mut Option<ExprTerm<'a>>) {
        debug!("and - {:?} : {:?}", &self, &other);
        let _ = ret.take();
        let tmp = self.cmp(other, &CmpAnd, &CmpAnd);
        debug!("and = {:?}", tmp);
        *ret = Some(tmp);
    }

    fn or(&self, other: &Self, ret: &mut Option<ExprTerm<'a>>) {
        debug!("or - {:?} : {:?}", &self, &other);
        let _ = ret.take();
        let tmp = self.cmp(other, &CmpOr, &CmpOr);
        debug!("or = {:?}", tmp);
        *ret = Some(tmp);
    }
}

impl<'a> Into<ExprTerm<'a>> for &Vec<&'a Value> {
    fn into(self) -> ExprTerm<'a> {
        if self.len() == 1 {
            match &self[0] {
                Value::Number(v) => return ExprTerm::Number(v.clone()),
                Value::String(v) => return ExprTerm::String(v.clone()),
                Value::Bool(v) => return ExprTerm::Bool(*v),
                _ => {}
            }
        }

        ExprTerm::Json(None, self.to_vec())
    }
}

fn walk_all_with_str<'a>(
    vec: &Vec<&'a Value>,
    tmp: &mut Vec<&'a Value>,
    key: &str,
    is_filter: bool,
) {
    if is_filter {
        walk(vec, tmp, &|v| match v {
            Value::Object(map) if map.contains_key(key) => Some(vec![v]),
            _ => None,
        });
    } else {
        walk(vec, tmp, &|v| match v {
            Value::Object(map) => match map.get(key) {
                Some(v) => Some(vec![v]),
                _ => None,
            },
            _ => None,
        });
    }
}

fn walk_all<'a>(vec: &Vec<&'a Value>, tmp: &mut Vec<&'a Value>) {
    walk(vec, tmp, &|v| match v {
        Value::Array(vec) => Some(vec.iter().collect()),
        Value::Object(map) => {
            let mut tmp = Vec::new();
            for (_, v) in map {
                tmp.push(v);
            }
            Some(tmp)
        }
        _ => None,
    });
}

fn walk<'a, F>(vec: &Vec<&'a Value>, tmp: &mut Vec<&'a Value>, fun: &F)
where
    F: Fn(&Value) -> Option<Vec<&Value>>,
{
    fn _walk<'a, F>(v: &'a Value, tmp: &mut Vec<&'a Value>, fun: &F)
    where
        F: Fn(&Value) -> Option<Vec<&Value>>,
    {
        if let Some(mut ret) = fun(v) {
            tmp.append(&mut ret);
        }

        match v {
            Value::Array(vec) => {
                for v in vec {
                    _walk(v, tmp, fun);
                }
            }
            Value::Object(map) => {
                for (_, v) in map {
                    _walk(&v, tmp, fun);
                }
            }
            _ => {}
        }
    }

    for v in vec {
        _walk(v, tmp, fun);
    }
}

fn abs_index(n: &isize, len: usize) -> usize {
    if n < &0_isize {
        (n + len as isize) as usize
    } else {
        *n as usize
    }
}

#[derive(Debug)]
enum FilterKey {
    String(String),
    All,
}

pub enum JsonPathError {
    EmptyPath,
    EmptyValue,
    Path(String),
    Serde(String),
}

impl fmt::Debug for JsonPathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for JsonPathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            JsonPathError::EmptyPath => f.write_str("path not set"),
            JsonPathError::EmptyValue => f.write_str("json value not set"),
            JsonPathError::Path(msg) => f.write_str(&format!("path error: \n{}\n", msg)),
            JsonPathError::Serde(msg) => f.write_str(&format!("serde error: \n{}\n", msg)),
        }
    }
}

#[derive(Debug)]
pub struct Selector<'a, 'b> {
    node: Option<Node>,
    node_ref: Option<&'b Node>,
    value: Option<&'a Value>,
    tokens: Vec<ParseToken>,
    terms: Vec<Option<ExprTerm<'a>>>,
    current: Option<Vec<&'a Value>>,
    selectors: Vec<Selector<'a, 'b>>,
}

impl<'a, 'b> Selector<'a, 'b> {
    pub fn new() -> Self {
        Selector {
            node: None,
            node_ref: None,
            value: None,
            tokens: Vec::new(),
            terms: Vec::new(),
            current: None,
            selectors: Vec::new(),
        }
    }

    pub fn str_path(&mut self, path: &str) -> Result<&mut Self, JsonPathError> {
        debug!("path : {}", path);

        if self.node_ref.is_some() {
            self.node_ref.take();
        }

        self.node = Some(Parser::compile(path).map_err(|e| JsonPathError::Path(e))?);
        Ok(self)
    }

    pub fn node_ref(&self) -> Option<&Node> {
        if let Some(node) = &self.node {
            Some(node)
        } else {
            None
        }
    }

    pub fn compiled_path(&mut self, node: &'b Node) -> &mut Self {
        if self.node.is_some() {
            self.node.take();
        }

        self.node_ref = Some(node);
        self
    }

    pub fn reset_value(&mut self) -> &mut Self {
        self.current = None;
        self
    }

    pub fn value(&mut self, v: &'a Value) -> &mut Self {
        self.value = Some(v);
        self
    }

    fn _select(&mut self) -> Result<(), JsonPathError> {
        if self.node_ref.is_some() {
            let node_ref = self.node_ref.take().unwrap();
            self.visit(node_ref);
            return Ok(());
        }

        if self.node.is_none() {
            return Err(JsonPathError::EmptyPath);
        }

        let node = self.node.take().unwrap();
        self.visit(&node);
        self.node = Some(node);

        Ok(())
    }

    pub fn select_as<T: serde::de::DeserializeOwned>(&mut self) -> Result<Vec<T>, JsonPathError> {
        self._select()?;

        match &self.current {
            Some(vec) => {
                let mut ret = Vec::new();
                for v in vec {
                    match T::deserialize(*v) {
                        Ok(v) => ret.push(v),
                        Err(e) => return Err(JsonPathError::Serde(e.to_string())),
                    }
                }
                Ok(ret)
            }
            _ => Err(JsonPathError::EmptyValue),
        }
    }

    pub fn select_as_str(&mut self) -> Result<String, JsonPathError> {
        self._select()?;

        match &self.current {
            Some(r) => {
                Ok(serde_json::to_string(r).map_err(|e| JsonPathError::Serde(e.to_string()))?)
            }
            _ => Err(JsonPathError::EmptyValue),
        }
    }

    pub fn select(&mut self) -> Result<Vec<&'a Value>, JsonPathError> {
        self._select()?;

        match &self.current {
            Some(r) => Ok(r.to_vec()),
            _ => Err(JsonPathError::EmptyValue),
        }
    }

    fn new_filter_context(&mut self) {
        self.terms.push(None);
        debug!("new_filter_context: {:?}", self.terms);
    }

    fn in_filter<F: Fn(&Vec<&'a Value>, &mut Vec<&'a Value>) -> FilterKey>(&mut self, fun: F) {
        match self.terms.pop() {
            Some(peek) => match peek {
                Some(v) => {
                    debug!("in_filter 1.: {:?}", v);

                    match v {
                        ExprTerm::Json(_, vec) => {
                            let mut tmp = Vec::new();
                            let filter_key = fun(&vec, &mut tmp);
                            self.terms.push(Some(ExprTerm::Json(Some(filter_key), tmp)));
                        }
                        _ => unreachable!(),
                    };
                }
                _ => {
                    debug!("in_filter 2.: {:?}", &self.current);

                    if let Some(current) = &self.current {
                        let mut tmp = Vec::new();
                        let filter_key = fun(current, &mut tmp);
                        self.terms.push(Some(ExprTerm::Json(Some(filter_key), tmp)));
                    }
                }
            },
            _ => {}
        }
    }

    fn all_in_filter_with_str(&mut self, key: &str) {
        self.in_filter(|vec, tmp| {
            walk_all_with_str(&vec, tmp, key, true);
            FilterKey::All
        });

        debug!("all_in_filter_with_str : {}, {:?}", key, self.terms);
    }

    fn next_in_filter_with_str(&mut self, key: &str) {
        fn _collect<'a>(
            v: &'a Value,
            tmp: &mut Vec<&'a Value>,
            key: &str,
            visited: &mut HashSet<*const Value>,
        ) {
            match v {
                Value::Object(map) => {
                    if map.contains_key(key) {
                        let ptr = v as *const Value;
                        if !visited.contains(&ptr) {
                            visited.insert(ptr);
                            tmp.push(v)
                        }
                    }
                }
                Value::Array(vec) => {
                    for v in vec {
                        _collect(v, tmp, key, visited);
                    }
                }
                _ => {}
            }
        }

        self.in_filter(|vec, tmp| {
            let mut visited = HashSet::new();
            for v in vec {
                _collect(v, tmp, key, &mut visited);
            }
            FilterKey::String(key.to_owned())
        });

        debug!("next_in_filter_with_str : {}, {:?}", key, self.terms);
    }

    fn next_from_current_with_num(&mut self, index: f64) {
        if let Some(current) = self.current.take() {
            let mut tmp = Vec::new();
            for c in current {
                if let Value::Array(vec) = c {
                    let index = abs_index(&(index as isize), vec.len());
                    if let Some(v) = c.get(index) {
                        tmp.push(v);
                    }
                }
            }
            self.current = Some(tmp);
        }

        debug!(
            "next_from_current_with_num : {:?}, {:?}",
            &index, self.current
        );
    }

    fn next_from_current_with_str(&mut self, keys: &Vec<String>) {
        if let Some(current) = self.current.take() {
            let mut tmp = Vec::new();
            for c in current {
                match c {
                    Value::Object(map) => {
                        for key in keys {
                            if let Some(v) = map.get(key) {
                                tmp.push(v)
                            }
                        }
                    }
                    _ => {}
                }
            }
            self.current = Some(tmp);
        }

        debug!(
            "next_from_current_with_str : {:?}, {:?}",
            keys, self.current
        );
    }

    fn next_all_from_current(&mut self) {
        if let Some(current) = self.current.take() {
            let mut tmp = Vec::new();
            for c in current {
                match c {
                    Value::Object(map) => {
                        for (_, v) in map {
                            tmp.push(v)
                        }
                    }
                    Value::Array(vec) => {
                        for v in vec {
                            tmp.push(v);
                        }
                    }
                    _ => {}
                }
            }
            self.current = Some(tmp);
        }

        debug!("next_all_from_current : {:?}", self.current);
    }

    fn all_from_current(&mut self) {
        if let Some(current) = self.current.take() {
            let mut tmp = Vec::new();
            walk_all(&current, &mut tmp);
            self.current = Some(tmp);
        }
        debug!("all_from_current: {:?}", self.current);
    }

    fn all_from_current_with_str(&mut self, key: &str) {
        if let Some(current) = self.current.take() {
            let mut tmp = Vec::new();
            walk_all_with_str(&current, &mut tmp, key, false);
            self.current = Some(tmp);
        }
        debug!("all_from_current_with_str: {}, {:?}", key, self.current);
    }
}

impl<'a, 'b> NodeVisitor for Selector<'a, 'b> {
    fn visit_token(&mut self, token: &ParseToken) {
        debug!("token: {:?}, stack: {:?}", token, self.tokens);

        if !self.selectors.is_empty() {
            match token {
                ParseToken::Absolute | ParseToken::Relative | ParseToken::Filter(_) => {
                    let selector = self.selectors.pop().unwrap();

                    if let Some(current) = &selector.current {
                        let term = current.into();

                        if let Some(s) = self.selectors.last_mut() {
                            s.terms.push(Some(term));
                        } else {
                            self.terms.push(Some(term));
                        }
                    } else {
                        unreachable!()
                    }
                }
                _ => {}
            }
        }

        if let Some(selector) = self.selectors.last_mut() {
            selector.visit_token(token);
            return;
        }

        match token {
            ParseToken::Absolute => {
                if self.current.is_some() {
                    let mut selector = Selector::new();

                    if let Some(value) = self.value {
                        selector.value = Some(value);
                        selector.current = Some(vec![value]);
                        self.selectors.push(selector);
                    }
                    return;
                }

                match &self.value {
                    Some(v) => self.current = Some(vec![v]),
                    _ => {}
                }
            }
            ParseToken::Relative => {
                if let Some(ParseToken::Array) = self.tokens.last() {
                    let array_token = self.tokens.pop();
                    if let Some(ParseToken::Leaves) = self.tokens.last() {
                        self.tokens.pop();
                        self.all_from_current();
                    }
                    self.tokens.push(array_token.unwrap());
                }
                self.new_filter_context();
            }
            ParseToken::In | ParseToken::Leaves | ParseToken::Array => {
                self.tokens.push(token.clone());
            }
            ParseToken::ArrayEof => {
                if let Some(Some(e)) = self.terms.pop() {
                    match e {
                        ExprTerm::Number(n) => {
                            self.next_from_current_with_num(to_f64(&n));
                        }
                        ExprTerm::String(key) => {
                            self.next_from_current_with_str(&vec![key]);
                        }
                        ExprTerm::Json(_, v) => {
                            if v.is_empty() {
                                self.current = Some(vec![&Value::Null]);
                            } else {
                                self.current = Some(v);
                            }
                        }
                        ExprTerm::Bool(false) => {
                            self.current = Some(vec![&Value::Null]);
                        }
                        _ => {}
                    }
                }

                self.tokens.pop();
            }
            ParseToken::All => {
                match self.tokens.last() {
                    Some(ParseToken::Array) => {
                        self.tokens.pop();
                    }
                    _ => {}
                }

                match self.tokens.last() {
                    Some(ParseToken::Leaves) => {
                        self.tokens.pop();
                        self.all_from_current();
                    }
                    Some(ParseToken::In) => {
                        self.tokens.pop();
                        self.next_all_from_current();
                    }
                    _ => {
                        self.next_all_from_current();
                    }
                }
            }
            ParseToken::Bool(b) => {
                self.terms.push(Some(ExprTerm::Bool(*b)));
            }
            ParseToken::Key(key) => {
                if let Some(ParseToken::Array) = self.tokens.last() {
                    self.terms.push(Some(ExprTerm::String(key.clone())));
                    return;
                }

                match self.tokens.pop() {
                    Some(t) => {
                        if self.terms.is_empty() {
                            match t {
                                ParseToken::Leaves => self.all_from_current_with_str(key.as_str()),
                                ParseToken::In => {
                                    self.next_from_current_with_str(&vec![key.clone()])
                                }
                                _ => {}
                            }
                        } else {
                            match t {
                                ParseToken::Leaves => {
                                    self.all_in_filter_with_str(key.as_str());
                                }
                                ParseToken::In => {
                                    self.next_in_filter_with_str(key.as_str());
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
            ParseToken::Keys(keys) => {
                if !self.terms.is_empty() {
                    unimplemented!("keys in filter");
                }

                if let Some(ParseToken::Array) = self.tokens.pop() {
                    self.next_from_current_with_str(keys);
                } else {
                    unreachable!();
                }
            }
            ParseToken::Number(v) => {
                self.terms
                    .push(Some(ExprTerm::Number(Number::from_f64(*v).unwrap())));
            }
            ParseToken::Filter(ref ft) => {
                let ref right = match self.terms.pop() {
                    Some(Some(right)) => right,
                    Some(None) => ExprTerm::Json(
                        None,
                        match &self.current {
                            Some(current) => current.to_vec(),
                            _ => unreachable!(),
                        },
                    ),
                    _ => panic!("empty term right"),
                };

                let left = match self.terms.pop() {
                    Some(Some(left)) => left,
                    Some(None) => ExprTerm::Json(
                        None,
                        match &self.current {
                            Some(current) => current.to_vec(),
                            _ => unreachable!(),
                        },
                    ),
                    _ => panic!("empty term left"),
                };

                let mut ret = None;
                match ft {
                    FilterToken::Equal => left.eq(right, &mut ret),
                    FilterToken::NotEqual => left.ne(right, &mut ret),
                    FilterToken::Greater => left.gt(right, &mut ret),
                    FilterToken::GreaterOrEqual => left.ge(right, &mut ret),
                    FilterToken::Little => left.lt(right, &mut ret),
                    FilterToken::LittleOrEqual => left.le(right, &mut ret),
                    FilterToken::And => left.and(right, &mut ret),
                    FilterToken::Or => left.or(right, &mut ret),
                };

                if let Some(e) = ret {
                    self.terms.push(Some(e));
                }
            }
            ParseToken::Range(from, to, step) => {
                if !self.terms.is_empty() {
                    unimplemented!("range syntax in filter");
                }

                if let Some(ParseToken::Array) = self.tokens.pop() {
                    let mut tmp = Vec::new();
                    if let Some(current) = &self.current {
                        for v in current {
                            if let Value::Array(vec) = v {
                                let from = if let Some(from) = from {
                                    abs_index(from, vec.len())
                                } else {
                                    0
                                };

                                let to = if let Some(to) = to {
                                    abs_index(to, vec.len())
                                } else {
                                    vec.len()
                                };

                                for i in (from..to).step_by(match step {
                                    Some(step) => *step,
                                    _ => 1,
                                }) {
                                    if let Some(v) = vec.get(i) {
                                        tmp.push(v);
                                    }
                                }
                            }
                        }
                    }
                    self.current = Some(tmp);
                } else {
                    unreachable!();
                }
            }
            ParseToken::Union(indices) => {
                if !self.terms.is_empty() {
                    unimplemented!("union syntax in filter");
                }

                if let Some(ParseToken::Array) = self.tokens.pop() {
                    let mut tmp = Vec::new();
                    if let Some(current) = &self.current {
                        for v in current {
                            if let Value::Array(vec) = v {
                                for i in indices {
                                    if let Some(v) = vec.get(abs_index(i, vec.len())) {
                                        tmp.push(v);
                                    }
                                }
                            }
                        }
                    }

                    self.current = Some(tmp);
                } else {
                    unreachable!();
                }
            }
            ParseToken::Eof => {
                debug!("visit_token eof");
            }
        }
    }
}

pub struct SelectorMut {
    path: Option<Node>,
    value: Option<Value>,
}

fn replace_value<F: FnMut(&Value) -> Value>(tokens: Vec<String>, value: &mut Value, fun: &mut F) {
    let mut target = value;

    for (i, token) in tokens.iter().enumerate() {
        let target_once = target;
        let is_last = i == tokens.len() - 1;
        let target_opt = match *target_once {
            Value::Object(ref mut map) => {
                if is_last {
                    let v = if let Some(v) = map.get(token) {
                        fun(v)
                    } else {
                        return;
                    };

                    map.insert(token.clone(), v);
                    return;
                }
                map.get_mut(token)
            }
            Value::Array(ref mut vec) => {
                if let Ok(x) = token.parse::<usize>() {
                    if is_last {
                        let v = { fun(&vec[x]) };
                        vec[x] = v;
                        return;
                    }
                    vec.get_mut(x)
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(t) = target_opt {
            target = t;
        } else {
            break;
        }
    }
}

impl SelectorMut {
    pub fn new() -> Self {
        SelectorMut {
            path: None,
            value: None,
        }
    }

    pub fn str_path(&mut self, path: &str) -> Result<&mut Self, JsonPathError> {
        self.path = Some(Parser::compile(path).map_err(|e| JsonPathError::Path(e))?);
        Ok(self)
    }

    pub fn value(&mut self, value: Value) -> &mut Self {
        self.value = Some(value);
        self
    }

    pub fn take(&mut self) -> Option<Value> {
        self.value.take()
    }

    fn compute_paths(&self, mut result: Vec<&Value>) -> Vec<Vec<String>> {
        fn _walk(
            origin: &Value,
            target: &mut Vec<&Value>,
            tokens: &mut Vec<String>,
            visited: &mut HashSet<*const Value>,
            visited_order: &mut Vec<Vec<String>>,
        ) -> bool {
            trace!("{:?}, {:?}", target, tokens);

            if target.is_empty() {
                return true;
            }

            target.retain(|t| {
                if std::ptr::eq(origin, *t) {
                    if visited.insert(*t) {
                        visited_order.push(tokens.to_vec());
                    }
                    false
                } else {
                    true
                }
            });

            match origin {
                Value::Array(vec) => {
                    for (i, v) in vec.iter().enumerate() {
                        tokens.push(i.to_string());
                        if _walk(v, target, tokens, visited, visited_order) {
                            return true;
                        }
                        tokens.pop();
                    }
                }
                Value::Object(map) => {
                    for (k, v) in map {
                        tokens.push(k.clone());
                        if _walk(v, target, tokens, visited, visited_order) {
                            return true;
                        }
                        tokens.pop();
                    }
                }
                _ => {}
            }

            return false;
        }

        let mut visited = HashSet::new();
        let mut visited_order = Vec::new();

        if let Some(origin) = &self.value {
            let mut tokens = Vec::new();
            _walk(
                origin,
                &mut result,
                &mut tokens,
                &mut visited,
                &mut visited_order,
            );
        }

        visited_order
    }

    pub fn delete(&mut self) -> Result<&mut Self, JsonPathError> {
        self.replace_with(&mut |_| Value::Null)
    }

    fn select(&self) -> Result<Vec<&Value>, JsonPathError> {
        if let Some(node) = &self.path {
            let mut selector = Selector::new();
            selector.compiled_path(&node);

            if let Some(value) = &self.value {
                selector.value(value);
            }

            Ok(selector.select()?)
        } else {
            Err(JsonPathError::EmptyPath)
        }
    }

    pub fn replace_with<F: FnMut(&Value) -> Value>(
        &mut self,
        fun: &mut F,
    ) -> Result<&mut Self, JsonPathError> {
        let paths = {
            let result = self.select()?;
            self.compute_paths(result)
        };

        if let Some(ref mut value) = &mut self.value {
            for tokens in paths {
                replace_value(tokens, value, fun);
            }
        }

        Ok(self)
    }
}
