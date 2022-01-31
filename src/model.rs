use std::fs::File;
use std::io::Read;

use glam::Vec3;

pub struct Model {
    verts: Vec<Vec3>,
    faces: Vec<Vec<usize>>,
}

impl Model {
    pub fn load_obj(path: &str) -> Model {
        let mut verts = Vec::new();
        let mut faces = Vec::new();

        let mut file = File::open(path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let mut lines = contents.lines();
        while let Some(line) = lines.next() {
            let mut words = line.split_whitespace();
            match words.next() {
                Some("v") => {
                    let x = words.next().unwrap().parse::<f32>().unwrap();
                    let y = words.next().unwrap().parse::<f32>().unwrap();
                    let z = words.next().unwrap().parse::<f32>().unwrap();
                    verts.push(Vec3::new(x, y, z));
                }
                Some("f") => {
                    let mut face = vec![];
                    for word in words {
                        let mut indices = word.split('/');
                        let v = indices.next().unwrap().parse::<usize>().unwrap() - 1;
                        face.push(v);
                    }
                    faces.push(face);
                }
                _ => {}
            }
        }
        Model { verts, faces }
    }

    pub fn nverts(&self) -> usize {
        self.verts.len()
    }

    pub fn nfaces(&self) -> usize {
        self.faces.len()
    }

    pub fn vert(&self, i: usize) -> Vec3 {
        self.verts[i]
    }

    pub fn face(&self, i: usize) -> &Vec<usize> {
        &self.faces[i]
    }
}
