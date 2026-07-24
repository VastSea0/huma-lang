use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum OpType {
    Leaf,
    Add(u64, u64),
    MatMul { left: u64, right: u64, r1: usize, c1: usize, c2: usize },
    ReLU(u64),
}

#[derive(Debug, Clone)]
pub struct TensorData {
    pub id: u64,
    pub satirlar: usize,
    pub sutunlar: usize,
    pub veri: Arc<Mutex<Vec<f64>>>,
    pub gradyan: Arc<Mutex<Vec<f64>>>,
    pub requires_grad: bool,
    pub op: OpType,
}

impl PartialEq for TensorData {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl fmt::Display for TensorData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tensor[{}×{}, id={}]", self.satirlar, self.sutunlar, self.id)
    }
}

pub struct AutogradGraph {
    next_id: u64,
    pub nodes: HashMap<u64, TensorData>,
}

impl AutogradGraph {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            nodes: HashMap::new(),
        }
    }

    pub fn tensor_olustur(&mut self, satirlar: usize, sutunlar: usize, veri: Vec<f64>, requires_grad: bool) -> TensorData {
        let id = self.next_id;
        self.next_id += 1;

        let size = satirlar * sutunlar;
        let t = TensorData {
            id,
            satirlar,
            sutunlar,
            veri: Arc::new(Mutex::new(veri)),
            gradyan: Arc::new(Mutex::new(vec![0.0; size])),
            requires_grad,
            op: OpType::Leaf,
        };

        self.nodes.insert(id, t.clone());
        t
    }

    pub fn topla(&mut self, t1: &TensorData, t2: &TensorData) -> TensorData {
        let id = self.next_id;
        self.next_id += 1;

        let v1 = t1.veri.lock().unwrap();
        let v2 = t2.veri.lock().unwrap();
        let size = v1.len();
        let mut res = vec![0.0; size];
        for i in 0..size {
            res[i] = v1[i] + v2[i];
        }

        let req_grad = t1.requires_grad || t2.requires_grad;
        let t = TensorData {
            id,
            satirlar: t1.satirlar,
            sutunlar: t1.sutunlar,
            veri: Arc::new(Mutex::new(res)),
            gradyan: Arc::new(Mutex::new(vec![0.0; size])),
            requires_grad: req_grad,
            op: OpType::Add(t1.id, t2.id),
        };

        self.nodes.insert(id, t.clone());
        t
    }

    pub fn matris_carp(&mut self, t1: &TensorData, t2: &TensorData) -> Result<TensorData, String> {
        if t1.sutunlar != t2.satirlar {
            return Err(format!("Matris boyut uyumsuzluğu: {}x{} * {}x{}", t1.satirlar, t1.sutunlar, t2.satirlar, t2.sutunlar));
        }

        let id = self.next_id;
        self.next_id += 1;

        let r1 = t1.satirlar;
        let c1 = t1.sutunlar;
        let c2 = t2.sutunlar;

        let v1 = t1.veri.lock().unwrap();
        let v2 = t2.veri.lock().unwrap();

        let mut res = vec![0.0; r1 * c2];
        for i in 0..r1 {
            for k in 0..c1 {
                let a = v1[i * c1 + k];
                for j in 0..c2 {
                    res[i * c2 + j] += a * v2[k * c2 + j];
                }
            }
        }

        let req_grad = t1.requires_grad || t2.requires_grad;
        let t = TensorData {
            id,
            satirlar: r1,
            sutunlar: c2,
            veri: Arc::new(Mutex::new(res)),
            gradyan: Arc::new(Mutex::new(vec![0.0; r1 * c2])),
            requires_grad: req_grad,
            op: OpType::MatMul { left: t1.id, right: t2.id, r1, c1, c2 },
        };

        self.nodes.insert(id, t.clone());
        Ok(t)
    }

    pub fn relu(&mut self, t1: &TensorData) -> TensorData {
        let id = self.next_id;
        self.next_id += 1;

        let v1 = t1.veri.lock().unwrap();
        let size = v1.len();
        let mut res = vec![0.0; size];
        for i in 0..size {
            res[i] = if v1[i] > 0.0 { v1[i] } else { 0.0 };
        }

        let t = TensorData {
            id,
            satirlar: t1.satirlar,
            sutunlar: t1.sutunlar,
            veri: Arc::new(Mutex::new(res)),
            gradyan: Arc::new(Mutex::new(vec![0.0; size])),
            requires_grad: t1.requires_grad,
            op: OpType::ReLU(t1.id),
        };

        self.nodes.insert(id, t.clone());
        t
    }

    pub fn backward(&mut self, target_id: u64) -> Result<(), String> {
        let target = match self.nodes.get(&target_id) {
            Some(n) => n.clone(),
            None => return Err(format!("Hedef tensor bulunamadı: {}", target_id)),
        };

        // Hedef gradyanını 1.0 ile doldur
        {
            let mut grad = target.gradyan.lock().unwrap();
            for g in grad.iter_mut() {
                *g = 1.0;
            }
        }

        // Topolojik sıralama yap
        let mut order = Vec::new();
        let mut visited = HashSet::new();
        self.topolojik_sirala(target_id, &mut visited, &mut order);
        order.reverse();

        for node_id in order {
            if let Some(node) = self.nodes.get(&node_id).cloned() {
                let node_grad = node.gradyan.lock().unwrap().clone();

                match node.op.clone() {
                    OpType::Leaf => {}
                    OpType::Add(left_id, right_id) => {
                        if let Some(left) = self.nodes.get(&left_id) {
                            let mut g = left.gradyan.lock().unwrap();
                            for i in 0..g.len() {
                                g[i] += node_grad[i];
                            }
                        }
                        if let Some(right) = self.nodes.get(&right_id) {
                            let mut g = right.gradyan.lock().unwrap();
                            for i in 0..g.len() {
                                g[i] += node_grad[i];
                            }
                        }
                    }
                    OpType::MatMul { left: left_id, right: right_id, r1, c1, c2 } => {
                        if let (Some(left), Some(right)) = (self.nodes.get(&left_id).cloned(), self.nodes.get(&right_id).cloned()) {
                            let v1 = left.veri.lock().unwrap();
                            let v2 = right.veri.lock().unwrap();

                            // dL/dLeft = dL/dOut * Right^T
                            let mut g_left = left.gradyan.lock().unwrap();
                            for i in 0..r1 {
                                for k in 0..c1 {
                                    let mut sum = 0.0;
                                    for j in 0..c2 {
                                        sum += node_grad[i * c2 + j] * v2[k * c2 + j];
                                    }
                                    g_left[i * c1 + k] += sum;
                                }
                            }

                            // dL/dRight = Left^T * dL/dOut
                            let mut g_right = right.gradyan.lock().unwrap();
                            for k in 0..c1 {
                                for j in 0..c2 {
                                    let mut sum = 0.0;
                                    for i in 0..r1 {
                                        sum += v1[i * c1 + k] * node_grad[i * c2 + j];
                                    }
                                    g_right[k * c2 + j] += sum;
                                }
                            }
                        }
                    }
                    OpType::ReLU(in_id) => {
                        if let Some(in_node) = self.nodes.get(&in_id) {
                            let v_in = in_node.veri.lock().unwrap();
                            let mut g_in = in_node.gradyan.lock().unwrap();
                            for i in 0..g_in.len() {
                                if v_in[i] > 0.0 {
                                    g_in[i] += node_grad[i];
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn topolojik_sirala(&self, curr_id: u64, visited: &mut HashSet<u64>, order: &mut Vec<u64>) {
        if visited.contains(&curr_id) { return; }
        visited.insert(curr_id);

        if let Some(node) = self.nodes.get(&curr_id) {
            match &node.op {
                OpType::Leaf => {}
                OpType::Add(l, r) => {
                    self.topolojik_sirala(*l, visited, order);
                    self.topolojik_sirala(*r, visited, order);
                }
                OpType::MatMul { left, right, .. } => {
                    self.topolojik_sirala(*left, visited, order);
                    self.topolojik_sirala(*right, visited, order);
                }
                OpType::ReLU(in_id) => {
                    self.topolojik_sirala(*in_id, visited, order);
                }
            }
        }
        order.push(curr_id);
    }
}

pub static AUTOGRAD_GRAF: once_cell::sync::Lazy<Arc<Mutex<AutogradGraph>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(AutogradGraph::new())));
