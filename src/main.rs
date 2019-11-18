#[macro_use]
extern crate glium;
//extern crate glium_text;
extern crate cgmath;
extern crate png;

extern crate rand;

mod framework;

use glium::{glutin, Surface, Display};
//use glium_text::{TextSystem, FontTexture};

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
    txcoord: [f32; 2],
}

implement_vertex!(Vertex, position, txcoord);

#[derive(Copy, Clone)]
struct Attribute {
    norm: [f32; 3],
    offset: [f32; 3],
}

implement_vertex!(Attribute, norm, offset);

struct Test {
    
    mesh: framework::mesh::Mesh<Vertex>,
    prog: Option<glium::program::Program>,
    texture: Option<glium::texture::Texture2d>,
    buffer: Option<glium::vertex::VertexBuffer<Attribute>>,
    param: glium::DrawParameters<'static>,
    //text_sys: TextSystem,
    //font: FontTexture,
    tick: u64,
}

impl Test {

    fn new() -> Self {
        // let vs = vec![
        //     Vertex { position: [-0.5, -0.5, -0.5], txcoord: [0.25, 0.5] },
        //     Vertex { position: [ 0.5, -0.5, -0.5], txcoord: [0.5, 0.5] },
        //     Vertex { position: [-0.5,  0.5, -0.5], txcoord: [0.25, 0.25] },
        //     Vertex { position: [ 0.5,  0.5, -0.5], txcoord: [0.5, 0.25] },
        //     Vertex { position: [-0.5, -0.5,  0.5], txcoord: [0.25, 0.75] },
        //     Vertex { position: [ 0.5, -0.5,  0.5], txcoord: [0.5, 0.75] },
        //     Vertex { position: [-0.5,  0.5,  0.5], txcoord: [0.25, 0.0] },
        //     Vertex { position: [ 0.5,  0.5,  0.5], txcoord: [0.5, 0.0] },
        // ];
        // let is = framework::mesh::INDICES8_BLOCK.to_vec();
        // let mesh = framework::mesh::Mesh::wrap(vs, is, glium::index::PrimitiveType::TrianglesList);
        
        let mut mesh = framework::mesh::Mesh::new();
        
        let f = [
            Vertex { position: [-0.5, -0.5, -0.5], txcoord: [0.25, 0.5] },
            Vertex { position: [ 0.5, -0.5, -0.5], txcoord: [0.5, 0.5] },
            Vertex { position: [-0.5,  0.5, -0.5], txcoord: [0.25, 0.25] },
            Vertex { position: [ 0.5,  0.5, -0.5], txcoord: [0.5, 0.25] },
        ];
        mesh.push(&f, &framework::mesh::INDICES4_RECT);

        let f = [
            Vertex { position: [-0.5, -0.5,  0.5], txcoord: [0.0, 0.5] },
            Vertex { position: [-0.5, -0.5, -0.5], txcoord: [0.25, 0.5] },
            Vertex { position: [-0.5,  0.5,  0.5], txcoord: [0.0, 0.25] },
            Vertex { position: [-0.5,  0.5, -0.5], txcoord: [0.25, 0.25] },
        ];
        mesh.push(&f, &framework::mesh::INDICES4_RECT);

        let f = [
            Vertex { position: [ 0.5, -0.5, -0.5], txcoord: [0.5, 0.5] },
            Vertex { position: [ 0.5, -0.5,  0.5], txcoord: [0.75, 0.5] },
            Vertex { position: [ 0.5,  0.5, -0.5], txcoord: [0.5, 0.25] },
            Vertex { position: [ 0.5,  0.5,  0.5], txcoord: [0.75, 0.25] },
        ];
        mesh.push(&f, &framework::mesh::INDICES4_RECT);

        let f = [
            Vertex { position: [ 0.5, -0.5,  0.5], txcoord: [0.75, 0.5] },
            Vertex { position: [-0.5, -0.5,  0.5], txcoord: [1.0, 0.5] },
            Vertex { position: [ 0.5,  0.5,  0.5], txcoord: [0.75, 0.25] },
            Vertex { position: [-0.5,  0.5,  0.5], txcoord: [1.0, 0.25] },
        ];
        mesh.push(&f, &framework::mesh::INDICES4_RECT);

        let f = [
            Vertex { position: [-0.5,  0.5, -0.5], txcoord: [0.25, 0.25] },
            Vertex { position: [ 0.5,  0.5, -0.5], txcoord: [0.5, 0.25] },
            Vertex { position: [-0.5,  0.5,  0.5], txcoord: [0.25, 0.0] },
            Vertex { position: [ 0.5,  0.5,  0.5], txcoord: [0.5, 0.0] },
        ];
        mesh.push(&f, &framework::mesh::INDICES4_RECT);

        let f = [
            Vertex { position: [-0.5, -0.5,  0.5], txcoord: [0.25, 0.75] },
            Vertex { position: [ 0.5, -0.5,  0.5], txcoord: [0.5, 0.75] },
            Vertex { position: [-0.5, -0.5, -0.5], txcoord: [0.25, 0.5] },
            Vertex { position: [ 0.5, -0.5, -0.5], txcoord: [0.5, 0.5] },
        ];
        mesh.push(&f, &framework::mesh::INDICES4_RECT);

        Test {
            mesh,
            prog: None,
            texture: None,
            buffer: None,
            param: glium::DrawParameters {
                depth: glium::Depth {
                    test: glium::draw_parameters::DepthTest::IfLess,
                    write: true,
                    .. Default::default()
                },
                .. Default::default()
            },
            tick: 0,
            
        }
    }
}

type GResult<T> = std::result::Result<T, Box<std::error::Error>>;

use framework::game::{GameLogic, Scheduler, SchedulerSettings};

impl GameLogic for Test {

    fn init(&mut self, display: &Display, settings: &mut SchedulerSettings) -> GResult<()> {
        settings.set_fps(60);
        settings.set_ups(50);
        let r = framework::util::Resource::default();
        let prog ={
            let vert = r.load_as_string("glsl/main.vert").map_err(Box::new)?;
            let frag = r.load_as_string("glsl/main.frag").map_err(Box::new)?;
            glium::Program::from_source(display, &vert, &frag, None).map_err(Box::new)?
        };
        self.prog = Some(prog);
        let img = framework::mesh::load_texture2d(r.open_read_only("rsc/crafting_table.png").map_err(Box::new)?)?;
        let texture = glium::texture::Texture2d::new(display, img).map_err(Box::new)?;
        self.texture = Some(texture);
        let buffer = {
            let m: i32 = 24;
            let n: i32 = 18;
            let mut attris = Vec::with_capacity((m * n) as usize);
            for i in (-m/2..(m+1)/2) {
                for j in (-n/2..(n+1)/2) {
                    let x = i as f32 * 1.9 / m as f32;
                    let y = j as f32 * 1.9 / n as f32;
                    attris.push(Attribute{ norm: [0.0, 0.0, 0.0], offset: [x, y, 0.0] } );
                }
            }
            glium::vertex::VertexBuffer::dynamic(display, attris.as_slice()).map_err(Box::new)?
        };
        self.buffer = Some(buffer);
        Ok(())
    }

    fn render(&mut self, dt: u64, display: &Display, settings: &mut SchedulerSettings) -> GResult<()> {
        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.1, 0.1, 0.4), 1.0);
        let prog = self.prog.as_ref().unwrap();
        let param = &self.param;
        let s = self.tick as f32 / 100.0;
        let texture = self.texture.as_ref().unwrap().sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest);
        let uniform = uniform!{texture_sampler: texture};
        let buffer = self.buffer.as_mut().unwrap();
        {
            
            let mut mapping = buffer.map();
            if self.tick % 2 == 0 {
                let cv: f32 = 0.02;
                for v in mapping.iter_mut() {
                    v.norm[0] += cv * (rand::random::<f32>() - 0.5);
                    v.norm[1] += cv * (rand::random::<f32>() - 0.5);
                    v.norm[2] += cv * (rand::random::<f32>() - 0.5);
                }
            } else {
                let cv: f32 = 0.01;
                for v in mapping.iter_mut() {
                    v.norm[0] += cv;
                    v.norm[1] += cv;
                    v.norm[2] += cv;
                }
            }
            
        }
        self.mesh.draw_instances(display, &mut target, buffer.per_instance().unwrap(), prog, &uniform, param).expect("err");
        
        target.finish().map_err(Box::new)?;
        //this.display.swap_buffers().map_err(Box::new)?;
        Ok(())
    }

    fn update(&mut self, dt: u64, settings: &mut SchedulerSettings) -> GResult<()> {
        self.tick += 1;
        Ok(())
    }

    fn handle_event(&mut self, event: glutin::Event, settings: &mut SchedulerSettings, close: &mut bool) -> GResult<()> {
        match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::CloseRequested => *close = true,
                _ => (),
            },
            _ => (),
        }
        Ok(())
    }

    fn finalize(&mut self, err: Option<Box<dyn std::error::Error>>) -> GResult<()> {
        if let Some(err) = err {
            println!("{}", err.as_ref());
        }
        Ok(())
    }
}

use std::io::Write;

fn main0() {

    let mut c = framework::spline::CubeSpline::new();
    c = c.compile(vec![1.0,2.0,4.0,5.0], vec![1.0,3.0,4.0,2.0]).unwrap();
    let mut f = std::fs::OpenOptions::new().write(true).create(true).open("test.csv").unwrap();
    let mut y = [0.0, 0.0, 0.0, 0.0];
    for i in (0..60) {
        y[0] = i as f64 / 100.0 * 10.0;
        c.get(y[0] , &mut y[1..2]);
        c.get_derivative(y[0], &mut y[2..3]);
        c.get_derivative2(y[0], &mut y[3..4]);
        write!(f, "{},{},{},{}\n", y[0], y[1], y[2], y[3]);
    }

}


use glium::glutin::{WindowBuilder, ContextBuilder};
use glium::glutin::dpi::LogicalSize;

type H = [u32; 8];

fn main() {
    
    for arg in std::env::args() {
        
    }
    let wb = WindowBuilder::new()
        .with_title("Example")
        .with_dimensions(LogicalSize::from((800,600)));
    let cb = ContextBuilder::new()
        .with_depth_buffer(8);
    let settings = SchedulerSettings::default();
    let g = Test::new();
    let c = framework::game::StdGameClock::default();
    let mut evtloop = Scheduler::new(wb, cb, settings, g, c);
    //let gl = Box::new(TestB{a:87});
    //let mut evtloop = evtloop.set_game_logic(gl);
    evtloop.run().expect("err");

}




