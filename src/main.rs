//#[macro_use]
extern crate glium;

use glium::implement_vertex;

fn main() {
    use glium::{glutin, Surface, uniform};
    
    let vertex_shader_src = r#"
        #version 330

        in vec2 position;
        uniform float t;
        uniform bool center_line;


        vec4 mod289(vec4 x)
        {
            return x - floor(x * (1.0 / 289.0)) * 289.0;
        }

        vec4 permute(vec4 x)
        {
            return mod289(((x*34.0)+10.0)*x);
        }

        vec4 taylorInvSqrt(vec4 r)
        {
            return 1.79284291400159 - 0.85373472095314 * r;
        }

        vec2 fade(vec2 t) {
            return t*t*t*(t*(t*6.0-15.0)+10.0);
        }

        // Classic Perlin noise
        float cnoise(vec2 P)
        {
            vec4 Pi = floor(P.xyxy) + vec4(0.0, 0.0, 1.0, 1.0);
            vec4 Pf = fract(P.xyxy) - vec4(0.0, 0.0, 1.0, 1.0);
            Pi = mod289(Pi); // To avoid truncation effects in permutation
            vec4 ix = Pi.xzxz;
            vec4 iy = Pi.yyww;
            vec4 fx = Pf.xzxz;
            vec4 fy = Pf.yyww;

            vec4 i = permute(permute(ix) + iy);

            vec4 gx = fract(i * (1.0 / 41.0)) * 2.0 - 1.0 ;
            vec4 gy = abs(gx) - 0.5 ;
            vec4 tx = floor(gx + 0.5);
            gx = gx - tx;

            vec2 g00 = vec2(gx.x,gy.x);
            vec2 g10 = vec2(gx.y,gy.y);
            vec2 g01 = vec2(gx.z,gy.z);
            vec2 g11 = vec2(gx.w,gy.w);

            vec4 norm = taylorInvSqrt(vec4(dot(g00, g00), dot(g01, g01), dot(g10, g10), dot(g11, g11)));
            g00 *= norm.x;  
            g01 *= norm.y;  
            g10 *= norm.z;  
            g11 *= norm.w;  

            float n00 = dot(g00, vec2(fx.x, fy.x));
            float n10 = dot(g10, vec2(fx.y, fy.y));
            float n01 = dot(g01, vec2(fx.z, fy.z));
            float n11 = dot(g11, vec2(fx.w, fy.w));

            vec2 fade_xy = fade(Pf.xy);
            vec2 n_x = mix(vec2(n00, n01), vec2(n10, n11), fade_xy.x);
            float n_xy = mix(n_x.x, n_x.y, fade_xy.y);
            return 2.3 * n_xy;
        }


        
        void main() {
            float noise_factor = 0.5;
            vec2 pos = position;
            float y;
            
            if (!center_line && gl_VertexID % 2 != 0) {
                y = cnoise(vec2(t+pos.x*noise_factor, 0.0));
            } else if (center_line) {
                y = cnoise(vec2(t+pos.x*noise_factor, 0.0));
            }
            pos.y += y;
            gl_Position = vec4(pos, 0.0, 1.0);
        }
    "#;

    let border_vertex_shader_src = r#"
        #version 330

        in vec2 position;

        void main() {
            gl_Position = vec4(position, 0.0, 1.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 330

        out vec4 color;
        uniform bool center_line;

        void main() {
            if (center_line) {
                color = vec4(0.3, 0.0, 0.0, 1.0);
            } else {
                color = vec4(1.0, 1.0, 1.0, 1.0);
            }
        }
    "#;
        
    
    /*
    *** Generate vertices ***
    */
    let padding = 0.1;
    let n_segments = 100;
    let center_line_width = 5.0;
    let lines_width = 1.0;
    let mut center_line: Vec::<Vertex> = Vec::new();
    let mut lines: Vec::<Vertex> = Vec::new();
    
    for i in 0..n_segments+1 {
        // Generate the center line:
        let x = map_range((0.0, n_segments as f32), (-1.0+padding, 1.0-padding), i as f32);
        center_line.push(Vertex {position: [x, 0.0]});

        // Generate the other lines:
        // Top to center left
        lines.push(Vertex {position: [x, 1.0-padding]});
        lines.push(Vertex {position: [-1.0+padding, 0.0]});

        // Bottom to center right
        lines.push(Vertex {position: [x, -1.0+padding]});
        lines.push(Vertex {position: [1.0-padding, 0.0]});
        
        // Top right to center
        lines.push(Vertex {position: [1.0-padding, 1.0-padding]});
        lines.push(Vertex {position: [x, 0.0]});

        // Bottom left to center
        lines.push(Vertex {position: [-1.0+padding, -1.0+padding]});
        lines.push(Vertex {position: [x, 0.0]});
    }

    // Border
    let border = vec![
        Vertex {position: [-1.0+padding, 1.0-padding]},
        Vertex {position: [1.0-padding, 1.0-padding]},
        Vertex {position: [1.0-padding, -1.0+padding]},
        Vertex {position: [-1.0+padding, -1.0+padding]},
    ];
    
    // Make screen and event loop
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    
    let center_line_vertex_buffer = glium::VertexBuffer::new(&display, &center_line).unwrap();
    let lines_vertex_buffer = glium::VertexBuffer::new(&display, &lines).unwrap();
    let border_vetex_buffer = glium::VertexBuffer::new(&display, &border).unwrap();
    let center_line_indices = glium::index::NoIndices(glium::index::PrimitiveType::LineStrip);
    let lines_indices = glium::index::NoIndices(glium::index::PrimitiveType::LinesList);
    let border_indices = glium::index::NoIndices(glium::index::PrimitiveType::LineLoop);

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();
    let border_program = glium::Program::from_source(&display, border_vertex_shader_src, fragment_shader_src, None).unwrap();
    
    let wide_line_params = glium::DrawParameters {
        line_width: Some(center_line_width),
        .. Default::default()
    };
    let narrow_line_params = glium::DrawParameters {
        line_width: Some(lines_width),
        .. Default::default()
    };

    // Time variable
    let mut t: f32 = 0.0;
    let mut center_line: bool = true;
    event_loop.run(move |ev, _, control_flow| {
        match ev {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                },
                _ => return,
            },
            glutin::event::Event::NewEvents(cause) => match cause {
                glutin::event::StartCause::ResumeTimeReached { .. } => (),
                glutin::event::StartCause::Init => (),
                _ => return,
            },
            _ => return,
        }
        
        let next_frame_time = std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);
        
        t += 0.01;
        
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        
        // Draw stuff here
        
        center_line = !center_line;
        target.draw(&lines_vertex_buffer, &lines_indices, &program, &uniform! {t: t, center_line: center_line}, &narrow_line_params).unwrap();
        center_line = !center_line;
        target.draw(&center_line_vertex_buffer, &center_line_indices, &program, &uniform! {t: t, center_line: center_line}, &wide_line_params).unwrap();
        
        target.draw(&border_vetex_buffer, &border_indices, &border_program, &glium::uniforms::EmptyUniforms, &narrow_line_params).unwrap();
        
        target.finish().unwrap();
        
    });
}


#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}

implement_vertex!(Vertex, position);


fn map_range(from_range: (f32, f32), to_range: (f32, f32), s: f32) -> f32 {
    to_range.0 + (s - from_range.0) * (to_range.1 - to_range.0) / (from_range.1 - from_range.0)
}