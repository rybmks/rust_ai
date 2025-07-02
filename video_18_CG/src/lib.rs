use dot_generator::{attr, node};
use dot_generator::{edge, id, node_id};
use dot_structures::*;

use std::{
    cell::RefCell,
    collections::{HashSet, LinkedList},
    ops::Deref,
    rc::Rc,
    sync::atomic::AtomicUsize,
};

#[derive(Debug, Clone)]
pub struct Value(Rc<Value_>);

#[derive(Debug, Clone)]
pub struct Value_ {
    id: usize,
    data: RefCell<f32>,
    grad: RefCell<f32>,
    op: Op,
}

fn topological_order(root: &Value) -> LinkedList<Value> {
    let mut order = LinkedList::new();
    let mut visited = HashSet::new();
    fn depth_first(value: &Value, visited: &mut HashSet<usize>, order: &mut LinkedList<Value>) {
        if !visited.contains(&value.id) {
            visited.insert(value.id);
            match &value.op {
                Op::None => {}
                Op::Add(lhs, rhs) | Op::Sub(lhs, rhs) | Op::Mul(lhs, rhs) => {
                    depth_first(lhs, visited, order);
                    depth_first(rhs, visited, order);
                }
                Op::Tanh(inner) => {
                    depth_first(inner, visited, order);
                }
            }
            order.push_front(value.clone());
        }
    }
    depth_first(root, &mut visited, &mut order);
    order
}

impl Value {
    pub fn new(data: f32) -> Self {
        Value(Rc::new(Value_::new(data)))
    }

    pub fn tanh(&self) -> Value {
        let d = self.get_data().tanh();
        let mut v = Value_::new(d);
        let op = Op::Tanh((*self).clone());
        v.op = op;
        Value(Rc::new(v))
    }
}
impl Deref for Value {
    type Target = Value_;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
static ID: AtomicUsize = AtomicUsize::new(1);

fn get_id() -> usize {
    ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

impl Value_ {
    pub fn new(data: f32) -> Self {
        Value_ {
            id: get_id(),
            data: RefCell::new(data),
            grad: RefCell::new(0.0),
            op: Op::None,
        }
    }
    pub fn get_data(&self) -> f32 {
        *self.data.borrow()
    }

    pub fn get_grad(&self) -> f32 {
        *self.grad.borrow()
    }
}

#[derive(Debug, Clone)]
enum Op {
    None,
    Add(Value, Value),
    Sub(Value, Value),
    Mul(Value, Value),
    Tanh(Value),
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Op::None => "None",
            Op::Add(_, _) => "Add",
            Op::Sub(_, _) => "Sub",
            Op::Mul(_, _) => "Mul",
            Op::Tanh(_) => "Tanh",
        };
        write!(f, "{name}",)
    }
}

impl std::ops::Add for &Value {
    type Output = Value;
    fn add(self, rhs: Self) -> Self::Output {
        let d = self.get_data() + rhs.get_data();
        let mut v = Value_::new(d);
        let op = Op::Add((*self).clone(), (*rhs).clone());
        v.op = op;
        Value(Rc::new(v))
    }
}

impl std::ops::Sub for &Value {
    type Output = Value;
    fn sub(self, rhs: Self) -> Self::Output {
        let d = self.get_data() - rhs.get_data();
        let mut v = Value_::new(d);
        let op = Op::Sub((*self).clone(), (*rhs).clone());
        v.op = op;
        Value(Rc::new(v))
    }
}

impl std::ops::Mul for &Value {
    type Output = Value;

    fn mul(self, rhs: Self) -> Self::Output {
        let d = self.get_data() * rhs.get_data();
        let mut v = Value_::new(d);
        let op = Op::Mul((*self).clone(), (*rhs).clone());
        v.op = op;
        Value(Rc::new(v))
    }
}

pub fn viz_computation_graph(value: &Value, graph: &mut Graph) {
    let tp_order = topological_order(value);
    for value in &tp_order {
        let value_node_id = value.id;
        let value_node = node!(
            value_node_id,
            vec![
                attr!("label", esc format!("{} | data={} | grad={} op={}", value.id,
    value.get_data(), value.get_grad(), value.op))
            ]
        );
        graph.add_stmt(value_node.into());
        let mut add_edge = |v: &Value| {
            let p_node_id = v.id;
            let e = edge!(node_id!(p_node_id) => node_id!(value_node_id));
            graph.add_stmt(e.into());
        };
        match &value.op {
            Op::Add(lhs, rhs) | Op::Sub(lhs, rhs) | Op::Mul(lhs, rhs) => {
                add_edge(lhs);
                add_edge(rhs);
            }
            Op::Tanh(inner) => {
                add_edge(inner);
            }
            Op::None => {}
        }
    }
}

pub fn calculat_grad(root: &Value) {
    *root.0.grad.borrow_mut() = 1.0;
    let tp_order = topological_order(root);
    for v in tp_order {
        match &v.op {
            Op::None => {}
            Op::Add(v1, v2) => {
                *v1.grad.borrow_mut() += v.get_grad();
                *v2.grad.borrow_mut() += v.get_grad();
            }
            Op::Mul(v1, v2) => {
                *v1.grad.borrow_mut() += v.get_grad() * v2.get_data();
                *v2.grad.borrow_mut() += v.get_grad() * v1.get_data();
            }
            Op::Tanh(v1) => {
                let t = v.get_data();
                let local_grad = 1.0 - t * t;
                let grad = v.get_grad() * local_grad;
                *v1.grad.borrow_mut() += grad;
            }
            Op::Sub(v1, v2) => {
                *v1.grad.borrow_mut() += v.get_grad();
                *v2.grad.borrow_mut() += -v.get_grad();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dot_generator::graph;
    use graphviz_rust::cmd::CommandArg;
    use graphviz_rust::{cmd::Format, exec, printer::PrinterContext};

    #[test]
    pub fn it_works() {
        let v1 = Value::new(0.5);
        let v2 = Value::new(0.3);
        let v3 = &v1 + &v2;
        let v4 = &v3 - &v2;
        let v5 = &v4 * &v2;
        let v6 = &v5.tanh();

        calculat_grad(v6);
        let mut graph = graph!(id!("copm"));
        viz_computation_graph(v6, &mut graph);
        let _graph_svg = exec(
            graph,
            &mut PrinterContext::default(),
            vec![Format::Png.into(), CommandArg::Output("./1.png".into())],
        )
        .unwrap();
    }
}
