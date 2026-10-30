//! Seletor HSV + alfa compartilhado pelo FeatherWorks.
//! Nasceu no FeatherSlide; não importa mensagens nem modelos de nenhum editor.
use pelicano::{button, column, container, program, row, text, text_input, Align, Background, Border, Captured, Color,
    Element, Event, Length, MouseButton, Path, Point, Program, Rect, Renderer, Shadow, Size, Space, Stroke, Style, Theme};
use std::f32::consts::FRAC_PI_2;
use crate::tema::*;

#[derive(Debug, Clone)]
pub enum Evento {
    SaturacaoBrilho(f32, f32), Matiz(f32), Alfa(f32),
    TextoHex(String), Canal(usize, String), Escolhida(String), Fechar,
}

/// A cor `#rrggbb` ou `#rrggbbaa` como [r, g, b, a]; vazia ou inválida vira preto opaco.
pub fn rgba_de(hex: &str) -> [u8; 4] {
    let s = hex.trim().trim_start_matches('#');
    let par = |i: usize| s.get(i..i + 2).and_then(|p| u8::from_str_radix(p, 16).ok());
    match s.len() {
        6 | 8 => [par(0).unwrap_or(0), par(2).unwrap_or(0), par(4).unwrap_or(0), if s.len() == 8 { par(6).unwrap_or(255) } else { 255 }],
        _ => [0, 0, 0, 255],
    }
}

/// `#rrggbb` se opaca, `#rrggbbaa` se tem transparência.
pub fn hex_de(c: [u8; 4]) -> String {
    if c[3] == 255 {
        format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])
    } else {
        format!("#{:02x}{:02x}{:02x}{:02x}", c[0], c[1], c[2], c[3])
    }
}

/// Só aceita o que `rgba_de` lê sem chute: 6 ou 8 dígitos hexadecimais.
pub fn hex_valido(s: &str) -> bool {
    let s = s.trim().trim_start_matches('#');
    (s.len() == 6 || s.len() == 8) && s.chars().all(|c| c.is_ascii_hexdigit())
}

/// h em graus (0..360), s e v em 0..1.
pub fn hsv_para_rgb(h: f32, s: f32, v: f32) -> [u8; 3] {
    let h = h.rem_euclid(360.0) / 60.0;
    let (c, i) = (v * s, h.floor() as i32);
    let x = c * (1.0 - (h % 2.0 - 1.0).abs());
    let (r, g, b) = match i {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = v - c;
    [((r + m) * 255.0).round() as u8, ((g + m) * 255.0).round() as u8, ((b + m) * 255.0).round() as u8]
}

/// [h graus, s, v, a 0..1]; `matiz_antes` mantém o matiz quando a cor é cinza (sem matiz próprio).
pub fn rgba_para_hsva(c: [u8; 4], matiz_antes: f32) -> [f32; 4] {
    let (r, g, b) = (c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0);
    let (mx, mn) = (r.max(g).max(b), r.min(g).min(b));
    let d = mx - mn;
    let h = if d < 1e-6 {
        matiz_antes
    } else if mx == r {
        60.0 * (((g - b) / d) % 6.0)
    } else if mx == g {
        60.0 * ((b - r) / d + 2.0)
    } else {
        60.0 * ((r - g) / d + 4.0)
    };
    [h.rem_euclid(360.0), if mx <= 0.0 { 0.0 } else { d / mx }, mx, c[3] as f32 / 255.0]
}

/// Aplica um evento do painel ao estado do app (HSVA e o texto do campo hex) e devolve a cor
/// nova (`#rrggbb` ou `#rrggbbaa`) quando ela muda; `Fechar` e entradas inválidas dão `None`.
pub fn aplicar(evento: Evento, hsva: &mut [f32; 4], hex: &mut String) -> Option<String> {
    let do_hsva = |h: &[f32; 4]| {
        let [r, g, b] = hsv_para_rgb(h[0], h[1], h[2]);
        hex_de([r, g, b, (h[3] * 255.).round() as u8])
    };
    let nova = match evento {
        Evento::Fechar => return None,
        // Vazia = sem cor: o estado fica como estava.
        Evento::Escolhida(cor) if cor.is_empty() => return Some(cor),
        Evento::Escolhida(cor) => cor,
        Evento::SaturacaoBrilho(s, v) => { hsva[1] = s.clamp(0., 1.); hsva[2] = v.clamp(0., 1.); do_hsva(hsva) }
        Evento::Matiz(h) => { hsva[0] = h.rem_euclid(360.); do_hsva(hsva) }
        Evento::Alfa(a) => { hsva[3] = a.clamp(0., 1.); do_hsva(hsva) }
        Evento::TextoHex(valor) => {
            *hex = valor.clone();
            if !hex_valido(&valor) { return None; }
            format!("#{}", valor.trim().trim_start_matches('#').to_lowercase())
        }
        Evento::Canal(i, valor) => {
            let n = valor.trim().parse::<u8>().ok().filter(|_| i < 4)?;
            let mut rgba = rgba_de(&do_hsva(hsva));
            rgba[i] = n;
            hex_de(rgba)
        }
    };
    if !hex_valido(&nova) { return None; }
    *hsva = rgba_para_hsva(rgba_de(&nova), hsva[0]);
    *hex = nova.clone();
    Some(nova)
}

/// Traduz o arrasto do mouse na área em (x, y) de 0 a 1; `None` se o evento não conta.
fn arrasto(arrastando: &mut bool, ev: &Event, b: Rect, c: Option<Point>) -> Option<(f32, f32)> {
    let norm = |p: Point| (((p.x - b.x) / b.width).clamp(0.0, 1.0), ((p.y - b.y) / b.height).clamp(0.0, 1.0));
    match ev {
        Event::MousePressed(MouseButton::Left) if c.is_some_and(|p| b.contains(p)) => {
            *arrastando = true;
            c.map(norm)
        }
        Event::CursorMoved(p) if *arrastando => Some(norm(*p)),
        Event::MouseReleased(MouseButton::Left) => {
            *arrastando = false;
            None
        }
        _ => None,
    }
}

fn degrade(r: &mut Renderer, area: Rect, angulo: f32, paradas: Vec<(f32, Color)>) {
    r.fill_quad(area, Some(&Background::Linear { angle: angulo, stops: paradas }), Border::default(), None);
}

fn cheio(r: &mut Renderer, area: Rect, c: Color) {
    r.fill_quad(area, Some(&c.into()), Border::default(), None);
}

fn marcador(r: &mut Renderer, p: Point) {
    if let Some(c) = Path::circle(p, 6.0) {
        r.stroke_path(&c, &Stroke::new(3.0), Color::BLACK);
        r.stroke_path(&c, &Stroke::new(1.5), Color::WHITE);
    }
}

/// A alça das barras: faixa branca com fio preto.
fn alca(r: &mut Renderer, area: Rect) {
    r.fill_quad(area, Some(&Color::WHITE.into()), Border::default().color(Color::BLACK, 1.0), None);
}

/// O quadrado: saturação para a direita, brilho para cima.
struct Quadrado {
    hsva: [f32; 4],
}

impl Program<Evento> for Quadrado {
    type State = bool;

    fn update(&self, estado: &mut bool, ev: &Event, b: Rect, c: Option<Point>, publicar: &mut dyn FnMut(Evento)) -> Captured {
        match arrasto(estado, ev, b, c) {
            Some((x, y)) => {
                publicar(Evento::SaturacaoBrilho(x, 1.0 - y));
                Captured::Yes
            }
            None => Captured::No,
        }
    }

    fn draw(&self, _: &bool, r: &mut Renderer, b: Rect, _: &Theme, _: Option<Point>) {
        let [vr, vg, vb] = hsv_para_rgb(self.hsva[0], 1.0, 1.0);
        degrade(r, b, 0.0, vec![(0.0, Color::WHITE), (1.0, Color::rgb8(vr, vg, vb))]);
        degrade(r, b, FRAC_PI_2, vec![(0.0, Color::TRANSPARENT), (1.0, Color::BLACK)]);
        marcador(r, Point::new(b.x + self.hsva[1] * b.width, b.y + (1.0 - self.hsva[2]) * b.height));
    }
}

/// A barra vertical do matiz (as cores do arco-íris).
struct BarraMatiz {
    hsva: [f32; 4],
}

impl Program<Evento> for BarraMatiz {
    type State = bool;

    fn update(&self, estado: &mut bool, ev: &Event, b: Rect, c: Option<Point>, publicar: &mut dyn FnMut(Evento)) -> Captured {
        match arrasto(estado, ev, b, c) {
            Some((_, y)) => {
                publicar(Evento::Matiz((y * 360.0).min(359.99)));
                Captured::Yes
            }
            None => Captured::No,
        }
    }

    fn draw(&self, _: &bool, r: &mut Renderer, b: Rect, _: &Theme, _: Option<Point>) {
        let paradas = (0..=6)
            .map(|i| {
                let [vr, vg, vb] = hsv_para_rgb(i as f32 * 60.0, 1.0, 1.0);
                (i as f32 / 6.0, Color::rgb8(vr, vg, vb))
            })
            .collect();
        degrade(r, b, FRAC_PI_2, paradas);
        let y = (self.hsva[0] / 360.0 * b.height - 2.0).max(0.0);
        alca(r, Rect::new(Point::new(b.x, b.y + y), Size::new(b.width, 4.0)));
    }
}

/// A barra horizontal de transparência, sobre um xadrez.
struct BarraAlfa {
    hsva: [f32; 4],
}

impl Program<Evento> for BarraAlfa {
    type State = bool;

    fn update(&self, estado: &mut bool, ev: &Event, b: Rect, c: Option<Point>, publicar: &mut dyn FnMut(Evento)) -> Captured {
        match arrasto(estado, ev, b, c) {
            Some((x, _)) => {
                publicar(Evento::Alfa(x));
                Captured::Yes
            }
            None => Captured::No,
        }
    }

    fn draw(&self, _: &bool, r: &mut Renderer, b: Rect, _: &Theme, _: Option<Point>) {
        xadrez(r, b);
        let [vr, vg, vb] = hsv_para_rgb(self.hsva[0], self.hsva[1], self.hsva[2]);
        let c1 = Color::rgb8(vr, vg, vb);
        degrade(r, b, 0.0, vec![(0.0, c1.alpha(0.0)), (1.0, c1)]);
        let x = (self.hsva[3] * b.width - 2.0).max(0.0);
        alca(r, Rect::new(Point::new(b.x + x, b.y), Size::new(4.0, b.height)));
    }
}

/// O xadrez cinza e branco que mostra o que é transparente.
fn xadrez(r: &mut Renderer, b: Rect) {
    let lado = 6.0;
    cheio(r, b, Color::WHITE);
    let (colunas, linhas) = ((b.width / lado).ceil() as i32, (b.height / lado).ceil() as i32);
    for l in 0..linhas {
        for c in 0..colunas {
            if (l + c) % 2 == 1 {
                let w = lado.min(b.width - c as f32 * lado);
                let h = lado.min(b.height - l as f32 * lado);
                cheio(r, Rect::new(Point::new(b.x + c as f32 * lado, b.y + l as f32 * lado), Size::new(w, h)), Color::rgb8(0xcc, 0xcc, 0xcc));
            }
        }
    }
}

fn cor(hex: &str) -> Color {
    let [r, g, b, a] = rgba_de(hex);
    Color::rgb8(r, g, b).alpha(a as f32 / 255.0)
}
const PRONTAS: [&str; 12] = [
    "#000000", "#666666", "#cccccc", "#ffffff", "#d93025", "#f29900",
    "#fbbc04", "#34a853", "#1a73e8", "#0b2545", "#9334e6", "#e52592",
];

/// Elemento reutilizável, sem posicionamento e sem estado próprio.
/// O aplicativo posiciona sobre sua Ribbon e mapeia Evento para a mensagem local.
pub fn painel<'a>(titulo: &'static str, atual: &str, hsva: [f32;4], texto_hex: &str) -> Element<'a, Evento> {
    painel_base(titulo, atual, hsva, texto_hex, false)
}

/// Versão integrada a diálogos de formatação: mesmos controles HSV/RGBA,
/// sem botão de fechar um popover que não existe nessa apresentação.
pub fn painel_embutido<'a>(titulo: &'static str, atual: &str, hsva: [f32;4], texto_hex: &str) -> Element<'a, Evento> {
    painel_base(titulo, atual, hsva, texto_hex, true)
}

fn painel_base<'a>(titulo: &'static str, atual: &str, hsva: [f32;4], texto_hex: &str, embutido: bool) -> Element<'a, Evento> {
    const LARGURA: f32 = 264.0;
    let rgba = rgba_de(atual);
    let quadrado = program(Quadrado { hsva }).width(Length::FILL).height(120.0);
    let matiz = program(BarraMatiz { hsva }).width(16.0).height(120.0);
    let alfa = program(BarraAlfa { hsva }).width(Length::FILL).height(14.0);

    let campo = |rotulo: &'static str, i: usize| -> Element<'a, Evento> {
        column![
            text_input("", rgba[i].to_string())
                .on_input(move |t| Evento::Canal(i, t))
                .size(12).padding([3, 4]).width(Length::FILL),
            text(rotulo).size(10).color(COR_TEXTO_MUTED()),
        ].spacing(2).align(Align::Center).width(Length::FILL).into()
    };

    let prontas = row(PRONTAS.iter().map(|hex| {
        let c = cor(hex);
        let escolhida = atual.eq_ignore_ascii_case(hex);
        Element::from(
            button(Space::new(16.0, 16.0))
                .padding(0)
                .style(move |_, _| Style::default().background(c).border(
                    Border::rounded(8.0).color(if escolhida { COR_AZUL_ACENTO() } else { COR_BORDA() }, if escolhida { 2.0 } else { 1.0 }),
                ))
                .on_press(Evento::Escolhida(hex.to_string()))
        )
    })).spacing(4);

    let mut rodape = row![
        button(text(crate::idioma::t("Sem cor")).size(12)).padding([5, 9])
            .style(estilo_botao_pequeno(atual.is_empty()))
            .on_press(Evento::Escolhida(String::new())),
    ].align(Align::Center).spacing(8);
    if !embutido {
        rodape = rodape.push(Space::new(Length::FILL, 0.0))
            .push(button(text(crate::idioma::t("Fechar")).size(12)).padding([5, 9])
                .style(estilo_botao_pequeno(false)).on_press(Evento::Fechar));
    }
    let conteudo = column![
        text(titulo).size(12).color(COR_TEXTO_MUTED()),
        row![quadrado, matiz].spacing(8).height(120.0),
        alfa,
        row![
            column![
                text_input("#rrggbb", texto_hex.to_owned()).on_input(Evento::TextoHex)
                    .size(12).padding([3, 4]).width(Length::FILL),
                text("Hex").size(10).color(COR_TEXTO_MUTED()),
            ].spacing(2).align(Align::Center).width(Length::Fill(3)),
            campo("R", 0), campo("G", 1), campo("B", 2), campo("A", 3),
        ].spacing(6),
        prontas,
        rodape,
    ].spacing(10);

    container(conteudo).padding(12)
        .width(LARGURA)
        .style(|_| Style::default()
            .background(COR_PAINEL())
            .text_color(COR_TEXTO())
            .border(Border::rounded(10.0).color(COR_BORDA(), 1.0))
            .shadow(Shadow { color: Color::BLACK.alpha(0.3), offset: Point::new(0.0, 6.0), blur: 18.0 }))
        .into()
}

#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn seletor_embutido_mantem_eventos_hsv() {
        let e: Element<'_, Evento> = painel_embutido("Cor", "#00ff00", [120.,1.,1.,1.], "#00ff00");
        let png = pelicano::capture(&e, Size::new(300.0, 420.0), 1.0, &Theme::light(), &mut Renderer::new(), None);
        assert!(!png.is_empty());
        let _ = painel("Cor", "#ff000080", [0., 1., 1., 0.5], "#ff000080");
    }

    fn arrastar<P: Program<Evento, State = bool>>(p: &P) -> Vec<Evento> {
        let b = Rect::new(Point::new(10.0, 20.0), Size::new(100.0, 50.0));
        let (mut st, mut fora) = (false, Vec::new());
        let mut pub_ = |e| fora.push(e);
        // Clique fora não começa arrasto; dentro começa; mover arrasta mesmo saindo da área.
        assert_eq!(p.update(&mut st, &Event::MousePressed(MouseButton::Left), b, Some(Point::new(0.0, 0.0)), &mut pub_), Captured::No);
        assert!(!st);
        assert_eq!(p.update(&mut st, &Event::MousePressed(MouseButton::Left), b, Some(Point::new(35.0, 30.0)), &mut pub_), Captured::Yes);
        assert!(st);
        p.update(&mut st, &Event::CursorMoved(Point::new(500.0, 500.0)), b, Some(Point::new(500.0, 500.0)), &mut pub_);
        p.update(&mut st, &Event::MouseReleased(MouseButton::Left), b, None, &mut pub_);
        assert!(!st);
        assert_eq!(p.update(&mut st, &Event::CursorMoved(Point::new(40.0, 40.0)), b, None, &mut pub_), Captured::No);
        fora
    }

    #[test]
    fn programas_arrastam_e_publicam() {
        let hsva = [0.0, 1.0, 1.0, 1.0];
        let q = arrastar(&Quadrado { hsva });
        assert!(matches!(q[..], [Evento::SaturacaoBrilho(x, y), Evento::SaturacaoBrilho(1.0, 0.0)]
            if (x - 0.25).abs() < 1e-6 && (y - 0.8).abs() < 1e-6));
        let m = arrastar(&BarraMatiz { hsva });
        assert!(matches!(m[..], [Evento::Matiz(h), Evento::Matiz(f)] if (h - 72.0).abs() < 1e-3 && f == 359.99));
        let a = arrastar(&BarraAlfa { hsva });
        assert!(matches!(a[..], [Evento::Alfa(x), Evento::Alfa(1.0)] if (x - 0.25).abs() < 1e-6));
    }
    #[test]
    fn aplicar_eventos_ao_estado() {
        let (mut hsva, mut hex) = ([0., 1., 1., 1.], String::from("#ff0000"));
        assert_eq!(aplicar(Evento::Canal(1, "255".into()), &mut hsva, &mut hex).as_deref(), Some("#ffff00"));
        assert_eq!(hex, "#ffff00");
        assert!((hsva[0] - 60.).abs() < 1e-3);
        assert_eq!(aplicar(Evento::TextoHex("#12".into()), &mut hsva, &mut hex), None);
        assert_eq!(hex, "#12");
        assert_eq!(aplicar(Evento::TextoHex("00FF00".into()), &mut hsva, &mut hex).as_deref(), Some("#00ff00"));
        assert_eq!(aplicar(Evento::Canal(0, "300".into()), &mut hsva, &mut hex), None);
        assert_eq!(aplicar(Evento::Escolhida(String::new()), &mut hsva, &mut hex).as_deref(), Some(""));
        assert_eq!(aplicar(Evento::Fechar, &mut hsva, &mut hex), None);
    }
    #[test]
    fn cores_hsv_transparencia_e_validacao() {
        assert_eq!(hsv_para_rgb(0.,1.,1.), [255,0,0]);
        assert_eq!(hex_de([255,0,0,255]), "#ff0000");
        assert_eq!(hex_de([255,0,0,128]), "#ff000080");
        assert!(hex_valido("#ff000080"));
        assert!(!hex_valido("#ff00gg"));
        let hsv=rgba_para_hsva([255,0,0,128],0.);
        assert_eq!(hsv[0],0.);
        assert!((hsv[3]-0.502).abs()<0.002);
    }
}
