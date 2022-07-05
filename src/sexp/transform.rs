use crate::sexp::{Error, SexpNode, SexpType};
use lazy_static::lazy_static;
use ndarray::{arr2, s, Array, Array1, Array2};
use std::collections::HashMap;

use crate::sexp::get::get;
use crate::sexp::get::SexpGet;

lazy_static! {
    pub static ref MIRROR: HashMap<String, Array2<f64>> = HashMap::from([ //TODO make global
        (String::from(""), arr2(&[[1., 0.], [0., -1.]])),
        (String::from("x"), arr2(&[[1., 0.], [0., 1.]])),
        (String::from("y"), arr2(&[[-1., 0.], [0., -1.]])),
        (String::from("xy"), arr2(&[[0., 0.], [0., 0.]]))
    ]);
}

/// transform the coordinates to absolute values.
pub trait Transform<T> {
    fn transform(&self, pts: &T) -> T;
}
impl Transform<Array2<f64>> for SexpNode {
    fn transform(&self, pts: &Array2<f64>) -> Array2<f64> {
        let pos: Array1<f64> = get!(self, "at");
        let angle: f64 = get!(self, "at", 2);
        let mirror: String = if self.contains("mirror") {
            //TODO use &str
            get!(self, "mirror", 0)
        } else {
            String::from("")
        };
        let theta = -angle.to_radians();
        let rot = arr2(&[[theta.cos(), -theta.sin()], [theta.sin(), theta.cos()]]);
        let mut verts: Array2<f64> = pts.dot(&rot);
        verts = verts.dot(MIRROR.get(mirror.as_str()).unwrap());
        let verts = pos + verts;
        verts.mapv_into(|v| format!("{:.2}", v).parse::<f64>().unwrap())
    }
}
impl Transform<Array1<f64>> for SexpNode {
    fn transform(&self, pts: &Array1<f64>) -> Array1<f64> {
        let pos: Array1<f64> = get!(self, "at");
        let angle: f64 = get!(self, "at", 2);
        let mirror: String = if self.contains("mirror") {
            //TODO use &str
            get!(self, "mirror", 0)
        } else {
            String::from("")
        };
        let theta = -angle.to_radians();
        let rot = arr2(&[[theta.cos(), -theta.sin()], [theta.sin(), theta.cos()]]);
        let mut verts: Array1<f64> = pts.dot(&rot);
        verts = verts.dot(MIRROR.get(mirror.as_str()).unwrap());
        let verts = pos + verts;
        verts.mapv_into(|v| format!("{:.2}", v).parse::<f64>().unwrap())
    }
}

/// transform the coordinates to absolute values.
pub trait Bounds<T> {
    fn bounds(&self, libs: &SexpNode) -> Result<T, Error>;
}
impl Bounds<Array2<f64>> for SexpNode {
    fn bounds(&self, libs: &SexpNode) -> Result<Array2<f64>, Error> {
        let mut boundery: Array2<f64> = Array2::default((0, 2));
        let _at: Array1<f64> = get!(self, "at");
        let _lib_id: String = get!(self, "lib_id", 0);

        for symbol in libs.nodes("symbol")? {
            if self.unit()? == symbol.unit()? || symbol.unit().unwrap() == 0 {
                let mut array = Vec::new();
                let mut rows: usize = 0;
                for element in &symbol.values {
                    if let SexpType::ChildSexpNode(element) = element {
                        if vec!["polyline"].contains(&element.name.as_str()) {
                            let pts: Array2<f64> = get!(element, "pts");
                            for row in pts.rows() {
                                let x = row[0].clone();
                                let y = row[1].clone();
                                array.extend_from_slice(&[x, y]);
                                rows += 1;
                            }
                        } else if vec!["rectangle"].contains(&element.name.as_str()) {
                            let start: Array1<f64> = get!(element, "start");
                            let end: Array1<f64> = get!(element, "end");
                            array.extend_from_slice(&[start[0], start[1]]);
                            array.extend_from_slice(&[end[0], end[1]]);
                            rows += 2;
                        } else if element.name != "pin" {
                            println!("Unknown: {:?}", element.name);
                        }
                    }
                }
                if rows > 0 {
                    let array = Array::from_shape_vec((rows, 2), array).unwrap();
                    let axis1 = array.slice(s![.., 0]);
                    let axis2 = array.slice(s![.., 1]);
                    boundery = arr2(&[
                        [
                            axis1
                                .iter()
                                .min_by(|a, b| a.partial_cmp(b).unwrap())
                                .unwrap()
                                .clone(),
                            axis2
                                .iter()
                                .min_by(|a, b| a.partial_cmp(b).unwrap())
                                .unwrap()
                                .clone(),
                        ],
                        [
                            axis1
                                .iter()
                                .max_by(|a, b| a.partial_cmp(b).unwrap())
                                .unwrap()
                                .clone(),
                            axis2
                                .iter()
                                .max_by(|a, b| a.partial_cmp(b).unwrap())
                                .unwrap()
                                .clone(),
                        ],
                    ]);
                }
            }
        }

        /* let axis1 = arr.slice(s![.., 0]);
        let axis2 = arr.slice(s![.., 1]);
        let boundery = arr2(&[
            [axis1.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
             axis2.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()],
            [axis1.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
             axis2.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()],
        ]); */

        Ok(boundery)
    }
}