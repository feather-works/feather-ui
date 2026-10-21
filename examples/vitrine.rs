//! Vitrine: tudo o que o pelicano e o feather-ui ganharam, numa tela só, para testar à mão.
//!
//! `cargo run --example vitrine` abre a janela; `-- --capturar saida.png` só gera o PNG (sem janela).
use feather_ui::ajuda::{self, Catalogo, Linha, Secao, Topico};
use feather_ui::caneta::{self, FontePressao, ToqueCaneta};
use feather_ui::campo_numero::{campo_numero, Limites};
use feather_ui::componentes::{botao_grande, botao_icone, lista_pesquisavel, lista_suspensa};
use feather_ui::cromo::{janela_com_barra, AcoesJanela};
use feather_ui::dialogo::{self as dlg, Dialogo};
use feather_ui::dropdown::{EstadoDropdown, EventoDropdown, RespostaDropdown};
use feather_ui::formatacao::{self, EstadoPopover};
use feather_ui::icones::icone;
use feather_ui::monitor::{self, Metricas, Monitor, Nivel};
use feather_ui::ribbon::{self, AbaRibbon, AcaoRibbon, EstadoMenuRibbon, EventoMenu, GrupoRibbon, ItemMenu};
use feather_ui::tema::*;
use feather_ui::tinta::{self, EventoCamada, Ferramenta, GestoTinta, TracoTela, Transformacao};
use feather_ui::{alinhamento, atalhos, configuracoes, creditos, efeitos_imagem, seletor_cor, texto_contorno};
use pelicano::column;
use pelicano::*;
use std::time::{Duration, Instant};

const MARKDOWN: &str = "### Markdown\n\
Texto **negrito**, *itálico*, ~~tachado~~ e `código`. Um [link](https://pelicano.dev) vai para o log.\n\n\
> Citação: clique no link acima.\n\n\
- [x] tarefa feita\n- [ ] tarefa por fazer\n\n\
| Widget | Estado |\n|---|---|\n| markdown | pronto |\n| tabela | pronta |\n\n\
```rust\nfn main() { println!(\"olá\"); }\n```\n";

const CODIGO: &str = "use pelicano::*;\n\n/// Realce pela feature `highlighter`.\nfn soma(a: i32, b: i32) -> i32 {\n    let total = a + b; // comentário\n    println!(\"{total}\");\n    total\n}\n";

const FONTES: [&str; 3] = ["Sans", "Serif", "Mono"];
const TAMANHOS: [u16; 5] = [10, 12, 14, 18, 24];
const OPCOES_DD: [&str; 5] = ["Árvore", "Banana", "Café", "Uva", "Uva verde"];
const FORMATOS: [&str; 4] = [".xlsx", ".ods", ".csv", ".fsdz"];
const SEGUNDA: WindowId = WindowId(1);

// Os campos só são lidos pelo Debug, que vira a linha de log.
#[allow(dead_code)]
#[derive(Debug, Clone)]
enum Msg {
    // janela
    Arrastar,
    Minimizar,
    Maximizar,
    Fechar,
    TelaCheia,
    AbrirSegunda,
    FecharSegunda,
    Evento(String),
    Escala(f32),
    Tamanho(Size),
    Atalho(String),
    Global(atalhos::AcaoGlobal),
    Esc,
    Escuro(bool),
    Tique,
    // pelicano
    Link(String),
    Rolou(Viewport),
    RolarPara,
    RolarFim,
    RolarInicio,
    Campo(String),
    Focar,
    Copiar,
    Colar,
    Colado(Option<String>),
    Forma(Point),
    Mouse(&'static str),
    MouseMove(Point),
    MouseRoda(f32, f32),
    Dica(TooltipPosition),
    Editado,
    TemaCodigo,
    Term(TerminalEvent),
    Mostrar(bool),
    Animado,
    Formato(&'static str),
    // feather-ui
    Cor(seletor_cor::Evento),
    Monitor(monitor::Evento),
    Onda(f32),
    AbrirDialogo,
    FecharDialogo,
    AbrirAjuda,
    Ajuda(ajuda::Evento),
    Popover(&'static str),
    CorTexto(String),
    Fonte(&'static str),
    TamanhoFonte(u16),
    Aba(u8),
    Ribbon(&'static str),
    Menu(EventoMenu<String>),
    AbrirMenu,
    Dropdown(EventoDropdown),
    Numero(String),
    Alinhar(alinhamento::Horizontal),
    Efeito(&'static str),
    Pesquisa(String),
    Combo(String),
    Lista(&'static str),
    Grande,
    Config(usize),
    TintaFaixa(tinta::Evento),
    Tinta(EventoCamada),
    Toques(Vec<ToqueCaneta>),
    Caneta(String),
}

struct Vitrine {
    ultima: String,
    contagem: u64,
    escala: f32,
    escuro: bool,
    ultimo_clique_barra: Option<Instant>,
    viewport: Option<Viewport>,
    campo: String,
    colado: String,
    forma: Point,
    editor: Content,
    tema_codigo: usize,
    terminal: Option<Terminal>,
    mostrar: bool,
    formato: Option<&'static str>,
    cor_hex: String,
    hsva: [f32; 4],
    texto_hex: String,
    monitor: Monitor,
    onda: f32,
    dialogo: bool,
    ajuda_aberta: bool,
    catalogo: Catalogo,
    ajuda: ajuda::Estado,
    popover: EstadoPopover,
    fonte: &'static str,
    tamanho_fonte: u16,
    aba: u8,
    menu: EstadoMenuRibbon,
    dropdown: EstadoDropdown,
    escolhido_dd: Option<usize>,
    numero: String,
    alinhamento: Option<alinhamento::Horizontal>,
    efeito: &'static str,
    pesquisaveis: Vec<String>,
    lista: Option<&'static str>,
    combo: Option<String>,
    config: usize,
    tinta: tinta::Estado,
    tracos: Vec<TracoTela>,
    proximo_traco: u64,
    /// Traço da caneta lida por fora (XInput2): quem o conduz é o app.
    gesto_caneta: Option<GestoTinta>,
    pressao: FontePressao,
}

fn itens_menu() -> Vec<ItemMenu<String>> {
    vec![
        ItemMenu::acao("novo", "Novo", "Novo".into()),
        ItemMenu::submenu("exportar", "Exportar", vec![ItemMenu::acao("pdf", "PDF", "PDF".into()), ItemMenu::acao("png", "PNG", "PNG".into())]),
        ItemMenu::inerte("inerte", "Sem ação").desabilitado(),
    ]
}

fn catalogo() -> Catalogo {
    let topico = |id: &str, titulo: &str, linhas: Vec<Linha>| Topico { id: id.into(), titulo: titulo.into(), linhas };
    Catalogo {
        titulo: "Ajuda da vitrine".into(),
        busca_rotulo: "Buscar na ajuda".into(),
        nada_encontrado: "Nada encontrado.".into(),
        anterior: "Anterior".into(),
        proximo: "Próximo".into(),
        nesta_secao: "Nesta seção".into(),
        secoes: vec![
            Secao {
                id: "inicio".into(),
                titulo: "Começando".into(),
                introducao: "Cada cartão testa um recurso. O log no alto mostra a última mensagem.".into(),
                topicos: vec![
                    topico("atalhos", "Atalhos", vec![
                        Linha::Atalho { descricao: "Abrir esta ajuda".into(), tecla: "F1".into() },
                        Linha::Atalho { descricao: "Qualquer Ctrl+tecla vai para o log".into(), tecla: "Ctrl+…".into() },
                        Linha::Atalho { descricao: "Fechar diálogos".into(), tecla: "Esc".into() },
                    ]),
                    topico("tinta", "Tinta", vec![Linha::Texto("Escolha uma caneta na faixa e desenhe no quadro.".into()), Linha::Marcador("A borracha apaga traços inteiros.".into())]),
                ],
            },
            Secao {
                id: "codigo".into(),
                titulo: "Código".into(),
                introducao: "O editor realça Rust pela feature highlighter.".into(),
                topicos: vec![topico("exemplo", "Exemplo", vec![Linha::Exemplo("cargo run --example vitrine".into())])],
            },
        ],
    }
}

impl Vitrine {
    fn nova(com_terminal: bool) -> Self {
        let escuro = system_dark();
        definir_tema(escuro, 0);
        let mut monitor = Monitor::novo("Vitrine");
        monitor.amostrar();
        monitor.registrar(Nivel::Info, "vitrine", "Vitrine aberta");
        let cor_hex = "#1a73e8".to_string();
        let hsva = seletor_cor::rgba_para_hsva(seletor_cor::rgba_de(&cor_hex), 0.0);
        Vitrine {
            ultima: "Nada ainda: mexa em qualquer coisa.".into(),
            contagem: 0,
            escala: 1.0,
            escuro,
            ultimo_clique_barra: None,
            viewport: None,
            campo: String::new(),
            colado: String::new(),
            forma: Point::new(60.0, 60.0),
            editor: Content::with_text(CODIGO),
            tema_codigo: 0,
            terminal: if com_terminal { Terminal::new(TerminalSettings::default()).ok() } else { None },
            mostrar: true,
            formato: None,
            texto_hex: cor_hex.clone(),
            cor_hex,
            hsva,
            monitor,
            onda: 10.0,
            dialogo: false,
            ajuda_aberta: false,
            catalogo: catalogo(),
            ajuda: ajuda::Estado::default(),
            popover: EstadoPopover::default(),
            fonte: FONTES[0],
            tamanho_fonte: 14,
            aba: 0,
            menu: EstadoMenuRibbon::default(),
            dropdown: EstadoDropdown::novo(),
            escolhido_dd: None,
            numero: "10".into(),
            alinhamento: Some(alinhamento::Horizontal::TODOS[0]),
            efeito: "original",
            pesquisaveis: ["Arial", "Calibri", "Comfortaa", "Liberation Serif", "Noto Sans", "Ubuntu"].map(String::from).to_vec(),
            lista: None,
            combo: None,
            config: 0,
            tinta: tinta::Estado::default(),
            tracos: Vec::new(),
            proximo_traco: 1,
            gesto_caneta: None,
            pressao: FontePressao::nova(),
        }
    }

    fn cor_do_hsva(&mut self) {
        let [r, g, b] = seletor_cor::hsv_para_rgb(self.hsva[0], self.hsva[1], self.hsva[2]);
        self.cor_hex = seletor_cor::hex_de([r, g, b, (self.hsva[3] * 255.0).round() as u8]);
        self.texto_hex = self.cor_hex.clone();
    }

    fn cor_do_hex(&mut self, hex: &str) {
        self.cor_hex = hex.to_string();
        self.hsva = seletor_cor::rgba_para_hsva(seletor_cor::rgba_de(hex), self.hsva[0]);
    }
}

fn cor_rgba(c: [u8; 4]) -> Color {
    Color::rgb8(c[0], c[1], c[2]).alpha(f32::from(c[3]) / 255.0)
}

/// Cartão com título, legenda do que testar e o conteúdo.
fn cartao<'a>(titulo: &str, legenda: &str, largura: f32, conteudo: impl Into<Element<'a, Msg>>) -> Element<'a, Msg> {
    container(
        column![
            text(titulo.to_owned()).size(15).font(Font::SEMIBOLD).color(COR_TEXTO()),
            text(legenda.to_owned()).size(12).color(COR_TEXTO_MUTED()),
            conteudo.into(),
        ]
        .spacing(8),
    )
    .padding(14)
    .width(largura)
    .style(|_| Style::default().background(COR_PAINEL()).border(Border::rounded(10.0).color(COR_BORDA(), 1.0)).text_color(COR_TEXTO()))
    .into()
}

fn botao<'a>(rotulo: &str, m: Msg) -> Element<'a, Msg> {
    button(text(rotulo.to_owned()).size(12)).padding([5u16, 10]).style(estilo_botao_secundario).on_press(m).into()
}

/// Círculo que se arrasta (program interativo).
struct Palco {
    pos: Point,
}

#[derive(Default)]
struct Arrasto {
    pega: Option<Point>,
}

impl Program<Msg> for Palco {
    type State = Arrasto;
    fn draw(&self, st: &Arrasto, r: &mut Renderer, b: Rect, _: &Theme, cursor: Option<Point>) {
        r.fill_quad(b, Some(&COR_MESA().into()), Border::rounded(8.0), None);
        let c = Point::new(b.x + self.pos.x, b.y + self.pos.y);
        let sobre = cursor.is_some_and(|p| (p.x - c.x).hypot(p.y - c.y) < 28.0);
        if let Some(p) = Path::circle(c, 28.0) {
            r.fill_path(&p, COR_AZUL_ACENTO().alpha(if st.pega.is_some() { 1.0 } else if sobre { 0.8 } else { 0.6 }), FillRule::NonZero);
        }
        r.draw_text("arraste", 11.0, Font::DEFAULT, COR_TEXTO_MUTED(), Point::new(b.x + 8.0, b.y + 6.0), None);
    }
    fn update(&self, st: &mut Arrasto, ev: &Event, b: Rect, cursor: Option<Point>, publish: &mut dyn FnMut(Msg)) -> Captured {
        let rel = cursor.map(|p| Point::new(p.x - b.x, p.y - b.y));
        match ev {
            Event::MousePressed(MouseButton::Left) => {
                if let Some(p) = rel.filter(|p| (p.x - self.pos.x).hypot(p.y - self.pos.y) < 28.0) {
                    st.pega = Some(Point::new(p.x - self.pos.x, p.y - self.pos.y));
                    return Captured::Yes;
                }
            }
            Event::CursorMoved(_) => {
                if let (Some(d), Some(p)) = (st.pega, rel) {
                    let x = (p.x - d.x).clamp(0.0, b.width);
                    let y = (p.y - d.y).clamp(0.0, b.height);
                    publish(Msg::Forma(Point::new(x, y)));
                    return Captured::Yes;
                }
            }
            Event::MouseReleased(MouseButton::Left) if st.pega.take().is_some() => return Captured::Yes,
            _ => {}
        }
        Captured::No
    }
    fn mouse_interaction(&self, st: &Arrasto, b: Rect, cursor: Option<Point>) -> CursorIcon {
        let perto = cursor.is_some_and(|p| (p.x - b.x - self.pos.x).hypot(p.y - b.y - self.pos.y) < 28.0);
        if st.pega.is_some() { CursorIcon::Grabbing } else if perto { CursorIcon::Grab } else { CursorIcon::Default }
    }
}

fn desenho_vetorial(r: &mut Renderer, a: Rect, _: &Theme) {
    r.fill_quad(a, Some(&COR_MESA().into()), Border::rounded(8.0), None);
    // Estrela com degradê.
    let (cx, cy) = (a.x + 70.0, a.y + 75.0);
    let estrela = Path::new(|b| {
        for i in 0..10 {
            let raio = if i % 2 == 0 { 55.0 } else { 24.0 };
            let ang = std::f32::consts::PI * (i as f32 / 5.0) - std::f32::consts::FRAC_PI_2;
            let p = Point::new(cx + raio * ang.cos(), cy + raio * ang.sin());
            if i == 0 { b.move_to(p); } else { b.line_to(p); }
        }
        b.close();
    });
    if let Some(e) = &estrela {
        r.fill_path(e, Background::Linear { angle: 0.8, stops: vec![(0.0, Color::rgb8(0xf9, 0x73, 0x16)), (1.0, Color::rgb8(0xec, 0x48, 0x99))] }, FillRule::NonZero);
        r.stroke_path(e, &Stroke::new(2.0).join(LineJoin::Round), COR_TEXTO());
    }
    // Curva tracejada com pontas redondas.
    let curva = Path::new(|b| {
        b.move_to(Point::new(a.x + 150.0, a.y + 120.0)).bezier_to(Point::new(a.x + 190.0, a.y + 10.0), Point::new(a.x + 250.0, a.y + 150.0), Point::new(a.x + 290.0, a.y + 40.0));
    });
    if let Some(c) = &curva {
        r.stroke_path(c, &Stroke::new(5.0).cap(LineCap::Round).dash(vec![12.0, 10.0]), COR_AZUL_ACENTO());
    }
    // Retângulo e texto girados.
    let centro = Point::new(a.x + 350.0, a.y + 80.0);
    r.push_transform(Transform::rotate_around(25.0, centro));
    if let Some(q) = Path::rect(Rect::new(Point::new(centro.x - 30.0, centro.y - 20.0), Size::new(60.0, 40.0))) {
        r.fill_path(&q, Color::rgb8(0x22, 0xc5, 0x5e), FillRule::NonZero);
    }
    r.fill_text("girado", 13.0, Font::BOLD, COR_TEXTO(), Point::new(centro.x - 22.0, centro.y + 26.0));
    r.pop_transform();
    // Contorno das letras (texto vazado).
    if let Some(t) = r.text_path("Vazado", 30.0, Font::BOLD, Point::new(a.x + 150.0, a.y + 128.0), None) {
        r.stroke_path(&t, &Stroke::new(1.2), Color::rgb8(0x93, 0x34, 0xe6));
    }
}

fn wordart(r: &mut Renderer, a: Rect, amplitude: f32) {
    r.fill_quad(a, Some(&COR_MESA().into()), Border::rounded(8.0), None);
    let x0 = a.x;
    let poligonos = texto_contorno::contornos_deformados(r, "Pelicano!", 52.0, Font::BOLD, Point::new(a.x + 20.0, a.y + 30.0), |[x, y]| {
        let t = (x - x0) / 60.0;
        [x, y + amplitude * t.sin() + (y - a.y - 60.0) * 0.15 * t.cos()]
    });
    let caminho = Path::new(|b| {
        for p in &poligonos {
            for (i, &[x, y]) in p.iter().enumerate() {
                if i == 0 { b.move_to(Point::new(x, y)); } else { b.line_to(Point::new(x, y)); }
            }
            b.close();
        }
    });
    if let Some(c) = caminho {
        r.fill_path(&c, Background::Linear { angle: 1.57, stops: vec![(0.0, Color::rgb8(0xfb, 0xbc, 0x04)), (1.0, Color::rgb8(0xd9, 0x30, 0x25))] }, FillRule::EvenOdd);
        r.stroke_path(&c, &Stroke::new(1.5).join(LineJoin::Round), COR_TEXTO());
    }
}

impl App for Vitrine {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) -> Task<Msg> {
        let ruidosa = matches!(msg, Msg::Tique | Msg::Term(TerminalEvent::Output) | Msg::MouseMove(_) | Msg::Rolou(_) | Msg::Forma(_) | Msg::Cor(_));
        if !matches!(msg, Msg::Tique | Msg::Term(TerminalEvent::Output)) {
            let mut s = format!("{msg:?}");
            s.truncate(s.char_indices().nth(180).map_or(s.len(), |(i, _)| i));
            self.contagem += 1;
            self.ultima = s;
            if !ruidosa {
                self.monitor.registrar(Nivel::Info, "vitrine", &self.ultima);
            }
        }
        match msg {
            Msg::Arrastar => return feather_ui::janelas::arrastar_ou_maximizar(WindowId::MAIN, &mut self.ultimo_clique_barra),
            Msg::Minimizar => return Task::window(WindowCmd::Minimize),
            Msg::Maximizar => return Task::window(WindowCmd::ToggleMaximize),
            Msg::Fechar => return Task::window(WindowCmd::Close),
            Msg::TelaCheia => return Task::window(WindowCmd::ToggleFullscreen),
            Msg::AbrirSegunda => return Task::open_window(SEGUNDA, Settings { title: "Segunda janela".into(), size: Size::new(420.0, 240.0), ..Default::default() }),
            Msg::FecharSegunda => return Task::window_of(SEGUNDA, WindowCmd::Close),
            Msg::Escala(s) => self.escala = s,
            Msg::Tamanho(t) => self.pressao.definir_tamanho_logico(t),
            Msg::Evento(_) | Msg::Atalho(_) | Msg::Link(_) | Msg::Mouse(_) | Msg::MouseMove(_) | Msg::MouseRoda(..) | Msg::Dica(_) | Msg::Editado | Msg::Caneta(_) | Msg::Grande | Msg::Ribbon(_) | Msg::Animado => {}
            Msg::Global(atalhos::AcaoGlobal::Ajuda) | Msg::AbrirAjuda => {
                self.ajuda_aberta = true;
                return ajuda::focar_busca();
            }
            Msg::Global(_) => {}
            Msg::Esc => {
                self.dialogo = false;
                self.ajuda_aberta = false;
                self.popover.fechar();
                self.menu.fechar();
                self.tinta.atualizar(tinta::Evento::Parar);
            }
            Msg::Escuro(e) => {
                self.escuro = e;
                definir_tema(e, 0);
            }
            Msg::Tique => {
                self.monitor.amostrar();
                return self.monitor.acompanhar();
            }
            Msg::Rolou(v) => self.viewport = Some(v),
            Msg::RolarPara => return Task::scroll_to("grade", Point::new(400.0, 300.0)),
            Msg::RolarFim => return Task::snap_to("grade", Point::new(1.0, 1.0)),
            Msg::RolarInicio => return Task::snap_to("grade", Point::ORIGIN),
            Msg::Campo(t) => self.campo = t,
            Msg::Focar => return Task::focus("campo-foco"),
            Msg::Copiar => return Task::write_clipboard(if self.campo.is_empty() { "copiado pela vitrine".to_string() } else { self.campo.clone() }),
            Msg::Colar => return Task::read_clipboard(Msg::Colado),
            Msg::Colado(t) => self.colado = t.unwrap_or_else(|| "(vazia)".into()),
            Msg::Forma(p) => self.forma = p,
            Msg::TemaCodigo => self.tema_codigo = (self.tema_codigo + 1) % Highlight::ALL.len(),
            Msg::Term(TerminalEvent::Exit(_)) => self.terminal = None,
            Msg::Term(_) => {}
            Msg::Mostrar(b) => self.mostrar = b,
            Msg::Formato(f) => self.formato = Some(f),
            Msg::Cor(ev) => match ev {
                seletor_cor::Evento::SaturacaoBrilho(s, v) => {
                    self.hsva[1] = s;
                    self.hsva[2] = v;
                    self.cor_do_hsva();
                }
                seletor_cor::Evento::Matiz(h) => {
                    self.hsva[0] = h;
                    self.cor_do_hsva();
                }
                seletor_cor::Evento::Alfa(a) => {
                    self.hsva[3] = a;
                    self.cor_do_hsva();
                }
                seletor_cor::Evento::TextoHex(t) => {
                    if seletor_cor::hex_valido(&t) {
                        self.cor_do_hex(&format!("#{}", t.trim().trim_start_matches('#')));
                    }
                    self.texto_hex = t;
                }
                seletor_cor::Evento::Canal(i, t) => {
                    if let Ok(v) = t.trim().parse::<u8>() {
                        let mut c = seletor_cor::rgba_de(&self.cor_hex);
                        c[i.min(3)] = v;
                        let hex = seletor_cor::hex_de(c);
                        self.cor_do_hex(&hex);
                        self.texto_hex = hex;
                    }
                }
                seletor_cor::Evento::Escolhida(h) => {
                    if !h.is_empty() {
                        self.cor_do_hex(&h);
                        self.texto_hex = h;
                    }
                    self.popover.fechar();
                }
                seletor_cor::Evento::Fechar => self.popover.fechar(),
            },
            Msg::Monitor(ev) => return self.monitor.atualizar(ev),
            Msg::Onda(v) => self.onda = v,
            Msg::AbrirDialogo => self.dialogo = true,
            Msg::FecharDialogo => self.dialogo = false,
            Msg::Ajuda(ev) => {
                if let ajuda::Evento::Copiar(t) = &ev {
                    let t = t.clone();
                    self.ajuda.atualizar(ev, &self.catalogo);
                    return Task::write_clipboard(t);
                }
                self.ajuda.atualizar(ev, &self.catalogo);
            }
            Msg::Popover(id) => self.popover.alternar(id),
            Msg::CorTexto(h) => {
                self.cor_do_hex(&h);
                self.texto_hex = h;
                self.popover.fechar();
            }
            Msg::Fonte(f) => self.fonte = f,
            Msg::TamanhoFonte(t) => self.tamanho_fonte = t,
            Msg::Aba(a) => self.aba = a,
            Msg::AbrirMenu => {
                if self.menu.aberto { self.menu.fechar() } else { self.menu.abrir(&itens_menu(), &[]); }
            }
            Msg::Menu(EventoMenu::Abrir(c)) => {
                self.menu.abrir(&itens_menu(), &c);
            }
            Msg::Menu(_) => self.menu.fechar(),
            Msg::Dropdown(ev) => {
                if let RespostaDropdown::Escolhido(i) = self.dropdown.atualizar(ev, &OPCOES_DD, self.escolhido_dd, false, 5) {
                    self.escolhido_dd = Some(i);
                }
            }
            Msg::Numero(t) => self.numero = t,
            Msg::Alinhar(a) => self.alinhamento = Some(a),
            Msg::Efeito(e) => self.efeito = e,
            Msg::Pesquisa(_) => {}
            Msg::Combo(c) => self.combo = Some(c),
            Msg::Lista(l) => self.lista = Some(l),
            Msg::Config(i) => self.config = i,
            Msg::TintaFaixa(ev) => {
                self.tinta.atualizar(ev);
            }
            Msg::Tinta(ev) => match ev {
                EventoCamada::TracoTerminado { ferramenta, pontos } => {
                    if let Some(p) = self.tinta.pena_de(ferramenta) {
                        self.tracos.push(TracoTela {
                            id: self.proximo_traco,
                            pontos: pontos.iter().map(|c| Point::new(c.x, c.y)).collect(),
                            cor: cor_rgba(p.cor),
                            espessura: p.espessura,
                            reto: false,
                            marca_texto: ferramenta == Ferramenta::MarcaTexto,
                            contorno: Vec::new(),
                        });
                        self.proximo_traco += 1;
                    }
                }
                EventoCamada::Apagar { ponto, raio, .. } => self.tracos.retain(|t| !t.pontos.iter().any(|p| (p.x - ponto.x).hypot(p.y - ponto.y) <= raio + t.espessura)),
                EventoCamada::ApagarFim | EventoCamada::Laco(_) => {}
            },
            Msg::Toques(lote) => {
                // Os toques na tinta não viram clique: o traço sai daqui.
                if let Some(f) = self.tinta.ferramenta_ativa() {
                    let origem = self.pressao.zona_tinta().map_or(Point::ORIGIN, |z| z.origem);
                    for t in &lote {
                        if let Some(ev) = GestoTinta::conduzir(&mut self.gesto_caneta, t, f, origem, Transformacao::default(), Some(&self.pressao)) {
                            let _ = self.update(Msg::Tinta(ev));
                        }
                    }
                } else {
                    self.gesto_caneta = None;
                }
                // VITRINE_CANETA_DIAG=1: diário da caneta no terminal.
                caneta::recolher_diagnostico().iter().for_each(|l| eprintln!("{l}"));
                return feather_ui::clique_caneta::entregar_cliques(WindowId::MAIN, &lote);
            }
        }
        Task::none()
    }

    fn theme(&self) -> Theme {
        tema()
    }

    fn on_event(&self, ev: &Event) -> Option<Msg> {
        match ev {
            Event::FileDropped(p) => Some(Msg::Evento(format!("arquivo solto: {}", p.display()))),
            Event::FileHovered(p) => Some(Msg::Evento(format!("arquivo em cima: {}", p.display()))),
            Event::FileHoverCancelled => Some(Msg::Evento("arrasto de arquivo cancelado".into())),
            Event::ModifiersChanged(m) => Some(Msg::Evento(format!("modificadores: {m:?}"))),
            Event::ScaleChanged(s) => Some(Msg::Escala(*s)),
            Event::Resized(t) => Some(Msg::Tamanho(*t)),
            Event::Focused(f) => Some(Msg::Evento(format!("foco da janela: {f}"))),
            // Toque: o primeiro dedo que ninguém pegar também chega como mouse; a tinta pega os dedos.
            Event::FingerPressed { id, position } => Some(Msg::Evento(format!("dedo {id} encostou em ({:.0}, {:.0})", position.x, position.y))),
            Event::FingerMoved { id, position } => Some(Msg::Evento(format!("dedo {id} em ({:.0}, {:.0})", position.x, position.y))),
            Event::FingerLifted { id, .. } => Some(Msg::Evento(format!("dedo {id} soltou"))),
            Event::FingerLost { id, .. } => Some(Msg::Evento(format!("dedo {id} cancelado pelo sistema"))),
            _ => None,
        }
    }

    fn on_unhandled_event(&self, id: WindowId, ev: &Event) -> Option<Msg> {
        let Event::Key { key, modifiers, pressed: true, .. } = ev else { return None };
        if id != WindowId::MAIN {
            return None;
        }
        if let Some(acao) = atalhos::acao_global(key, *modifiers) {
            return Some(Msg::Global(acao));
        }
        if matches!(key, Key::Named(NamedKey::Escape)) {
            return Some(Msg::Esc);
        }
        if modifiers.control_key() || modifiers.alt_key() {
            return atalhos::combo_de(key, *modifiers).map(Msg::Atalho);
        }
        None
    }

    fn subscription(&self) -> Subscription<Msg> {
        let mut subs = vec![Subscription::every(Duration::from_secs(1)).map(|_| Msg::Tique), caneta::assinatura_toques(&self.pressao).map(Msg::Toques)];
        if let Some(t) = &self.terminal {
            subs.push(t.subscription().map(Msg::Term));
        }
        Subscription::batch(subs)
    }

    fn view_window(&self, id: WindowId) -> Element<'_, Msg> {
        if id == WindowId::MAIN {
            return self.view();
        }
        container(column![text("Segunda janela").size(18), text("Mesmo app, outro WindowId.").color(COR_TEXTO_MUTED()), botao("Fechar esta janela", Msg::FecharSegunda)].spacing(10))
            .padding(20)
            .width(Length::FILL)
            .height(Length::FILL)
            .style(|_| Style::default().background(COR_FUNDO_JANELA()).text_color(COR_TEXTO()))
            .into()
    }

    fn view(&self) -> Element<'_, Msg> {
        let estreito = 440.0;
        let largo = 2.0 * estreito + 14.0;
        let mut cartoes: Vec<Element<'_, Msg>> = Vec::new();

        // ---------- pelicano ----------
        cartoes.push(cartao("Markdown", "Clique no link: o endereço aparece no log.", estreito, markdown(MARKDOWN, 13.0, Msg::Link)));

        cartoes.push(cartao(
            "rich_text e spans",
            "Estilos por trecho; o trecho-link manda mensagem.",
            estreito,
            rich_text([
                span("Texto "),
                span("negrito ").font(Font::BOLD),
                span("itálico ").italic(),
                span("sublinhado ").underline(),
                span("tachado ").strikethrough(),
                span(" marca-texto ").background(Color::rgb8(0xfb, 0xbc, 0x04).alpha(0.5)),
                span(" e um "),
                span("link").link(Msg::Link("span".into())),
                span(" com "),
                span("cor").color(COR_ERRO()).size(18.0),
                span("."),
            ])
            .size(14),
        ));

        let grade = column((0..30).map(|l| row((0..30).map(|c| container(text(format!("{l},{c}")).size(10)).center().width(60.0).height(30.0).style(move |_| Style::default().background(if (l + c) % 2 == 0 { COR_MESA() } else { COR_PAINEL() })).into())).into()));
        let vp = self.viewport.map_or("role a grade".into(), |v| {
            let rel = v.relative();
            format!("offset ({:.0}, {:.0}) · visível {:.0}×{:.0} · conteúdo {:.0}×{:.0} · rel ({:.2}, {:.2})", v.offset.x, v.offset.y, v.bounds.width, v.bounds.height, v.content.width, v.content.height, rel.x, rel.y)
        });
        cartoes.push(cartao(
            "scrollable (Direction::Both)",
            "Role nos dois eixos (Shift+roda = horizontal). Botões: scroll_to / snap_to pelo id.",
            estreito,
            column![
                container(scrollable(grade).direction(Direction::Both).id("grade").on_scroll(Msg::Rolou)).height(160.0).clip(true),
                text(vp).size(11).color(COR_TEXTO_MUTED()),
                row![botao("scroll_to (400,300)", Msg::RolarPara), botao("snap_to fim", Msg::RolarFim), botao("snap_to início", Msg::RolarInicio)].spacing(6),
            ]
            .spacing(6),
        ));

        cartoes.push(cartao(
            "Foco e área de transferência",
            "Focar põe o cursor no campo (Task::focus). Copiar/Colar usam a área do sistema.",
            estreito,
            column![
                text_input("Digite algo…", &self.campo).id("campo-foco").on_input(Msg::Campo),
                row![botao("Focar campo", Msg::Focar), botao("Copiar campo", Msg::Copiar), botao("Ler área", Msg::Colar)].spacing(6),
                text(format!("Lido: {}", if self.colado.is_empty() { "—" } else { &self.colado })).size(12),
                pick_list(&FORMATOS[..], self.formato, Msg::Formato).placeholder("pick_list…").width(160.0),
            ]
            .spacing(8),
        ));

        cartoes.push(cartao(
            "Eventos da janela",
            "Arraste um arquivo para a janela; aperte Ctrl/Shift/Alt; troque de monitor (escala); qualquer Ctrl+tecla vira atalho.",
            estreito,
            column![
                text(format!("Tema do sistema escuro: {} · escala: {:.2}", system_dark(), self.escala)).size(12),
                row![
                    botao("Tela cheia", Msg::TelaCheia),
                    botao("Maximizar", Msg::Maximizar),
                    botao("Segunda janela", Msg::AbrirSegunda),
                ]
                .spacing(6),
                toggler("Tema escuro (feather-ui)", self.escuro).on_toggle(Msg::Escuro),
            ]
            .spacing(8),
        ));

        cartoes.push(cartao("Vetores", "Path, degradê, traço tracejado, Transform::rotate_around, fill_text e text_path.", estreito, canvas(Length::FILL, 160.0, desenho_vetorial)));

        cartoes.push(cartao("program (canvas interativo)", "Arraste o círculo; a posição vai para o log.", estreito, program(Palco { pos: self.forma }).width(Length::FILL).height(160.0)));

        cartoes.push(cartao(
            "responsive e MouseArea",
            "O primeiro bloco mostra o próprio tamanho; no segundo, entre, saia, mova e role.",
            estreito,
            column![
                container(responsive(|s| container(text(format!("{:.0} × {:.0}", s.width, s.height)).size(13)).center().width(Length::FILL).height(Length::FILL).into())).height(50.0).style(|_| Style::default().background(COR_AZUL_FUNDO()).border(Border::rounded(6.0))),
                mouse_area(container(text("passe o mouse e role aqui").size(12)).center().width(Length::FILL).height(70.0).style(|_| Style::default().background(COR_MESA()).border(Border::rounded(6.0))))
                    .on_enter(Msg::Mouse("entrou"))
                    .on_exit(Msg::Mouse("saiu"))
                    .on_move(Msg::MouseMove)
                    .on_scroll(|dx, dy, _| Msg::MouseRoda(dx, dy))
                    .on_press(Msg::Mouse("clique")),
            ]
            .spacing(8),
        ));

        let dica = |rotulo: &'static str, pos: TooltipPosition| tooltip(button(text(rotulo).size(12)).padding([5u16, 10]).style(estilo_botao_secundario).on_press(Msg::Dica(pos)), format!("Dica em {rotulo}"), pos);
        cartoes.push(cartao(
            "Tooltips e acessibilidade",
            "Pare o mouse em cima (350 ms). O botão só de ícone tem access_label para o leitor de tela.",
            estreito,
            column![
                row![dica("Top", TooltipPosition::Top), dica("Bottom", TooltipPosition::Bottom), dica("Left", TooltipPosition::Left), dica("Right", TooltipPosition::Right)].spacing(8),
                Element::from(button(icone("settings", 18.0)).padding(6).style(estilo_botao_secundario).on_press(Msg::Mouse("ícone acessível"))).access_label("Configurações (rótulo acessível)"),
            ]
            .spacing(10),
        ));

        let tema_cod = Highlight::ALL[self.tema_codigo];
        cartoes.push(cartao(
            "text_editor com realce",
            "Feature highlighter (syntect). Edite; troque o tema de cores.",
            estreito,
            column![
                container(text_editor(&self.editor).highlight("rs", tema_cod).line_numbers(true).size(12.0).on_change(Msg::Editado)).height(170.0),
                row![botao("Trocar tema", Msg::TemaCodigo), text(format!("{tema_cod:?} · linha {} col {}", self.editor.cursor().0, self.editor.cursor().1)).size(11).color(COR_TEXTO_MUTED())].spacing(8).align(Align::Center),
            ]
            .spacing(6),
        ));

        let term: Element<'_, Msg> = match &self.terminal {
            Some(t) => terminal(t).size(12.0).into(),
            None => container(text("Terminal indisponível (captura ou shell encerrado).").size(12)).center().width(Length::FILL).height(Length::FILL).into(),
        };
        cartoes.push(cartao("Terminal embutido", "Feature terminal: seu $SHELL num PTY. Clique para focar; título e sino vão para o log.", estreito, container(term).height(200.0).clip(true)));

        let girando = animate(container(text("✦").size(28)).center().width(48.0).height(48.0).style(|_| Style::default().background(COR_AZUL_FUNDO()).border(Border::rounded(24.0))))
            .rotate(360.0)
            .repeat(Repeat::Forever)
            .transition(Transition::new(3000).easing(Easing::Linear));
        let saindo = animate(container(text("Eu entro e saio").size(13)).padding(10).style(|_| Style::default().background(COR_MESA()).border(Border::rounded(8.0))))
            .show(self.mostrar)
            .enter(Keyframe::default().opacity(0.0).offset(0.0, 16.0))
            .exit(Keyframe::default().opacity(0.0).scale(0.8))
            .hover(Keyframe::default().scale(1.05));
        let transicao = button(text("Hover com transição de 400 ms").size(12)).on_press(Msg::Animado).style(|t, s| button_primary(t, s).transition(Transition::new(400).easing(Easing::EaseOut)));
        cartoes.push(cartao(
            "Transições e animações",
            "Giro sem fim, entrada/saída com show(), hover que cresce e botão com transição.",
            estreito,
            column![row![girando, saindo].spacing(14).align(Align::Center), row![toggler("Mostrar", self.mostrar).on_toggle(Msg::Mostrar), transicao].spacing(12).align(Align::Center)].spacing(10),
        ));

        // ---------- feather-ui ----------
        let painel_cor = seletor_cor::painel_embutido("Seletor de cor", &self.cor_hex, self.hsva, &self.texto_hex).map(Msg::Cor);
        cartoes.push(cartao("seletor_cor", "Quadro S/V, matiz, alfa, canais e hex. A cor vale para a prévia da formatação.", estreito, painel_cor));

        cartoes.push(cartao(
            "monitor (F12)",
            "Memória amostrada a cada segundo; aba Console tem cada mensagem desta vitrine.",
            largo,
            container(self.monitor.view(Metricas { cartoes: vec![("Mensagens".into(), self.contagem.to_string(), "desde a abertura".into())], linhas: vec![("Traços de tinta".into(), self.tracos.len().to_string())], ocupado: None }, Msg::Monitor)).height(600.0).clip(true),
        ));

        let amp = self.onda;
        cartoes.push(cartao(
            "texto_contorno (WordArt)",
            "contornos_deformados com uma onda; mude a amplitude.",
            estreito,
            column![canvas(Length::FILL, 130.0, move |r, a, _| wordart(r, a, amp)), row![text("Onda").size(12), slider(0.0..=30.0, self.onda, Msg::Onda)].spacing(8).align(Align::Center)].spacing(6),
        ));

        let cor_popover: Option<Element<'_, Msg>> = self.popover.esta_aberto("cor").then(|| {
            let mut linha = row![].spacing(4);
            for h in ["#d93025", "#34a853", "#1a73e8", "#9334e6", "#202124"] {
                let c = Color::hex(h).unwrap_or(Color::BLACK);
                linha = linha.push(button(Space::new(18.0, 18.0)).padding(0).style(move |_, _| Style::default().background(c).border(Border::rounded(4.0))).on_press(Msg::CorTexto(h.into())));
            }
            formatacao::popover("Cor do texto", linha.into())
        });
        cartoes.push(cartao(
            "formatacao",
            "Fonte, tamanho, botão de cor com popover e prévia do estilo.",
            estreito,
            column![
                row![
                    formatacao::seletor_fonte(&FONTES[..], Some(self.fonte), Msg::Fonte, 90.0),
                    formatacao::seletor_tamanho(&TAMANHOS[..], Some(self.tamanho_fonte), Msg::TamanhoFonte, 70.0),
                    formatacao::botao_cor("Cor", &self.cor_hex, Some(Msg::Popover("cor")), self.popover.esta_aberto("cor")),
                    formatacao::botao_cor_compacto(formatacao::TipoCorCompacta::Texto, &self.cor_hex, Some(Msg::Popover("cor")), false),
                ]
                .spacing(6)
                .align(Align::Center),
            ]
            .push_maybe(cor_popover)
            .push(formatacao::previa_estilo(&format!("{} {} pt", self.fonte, self.tamanho_fonte), &self.cor_hex, f32::from(self.tamanho_fonte), true))
            .spacing(8),
        ));

        let abas_ribbon = [AbaRibbon::nova(0u8, "Início"), AbaRibbon::nova(1, "Inserir"), AbaRibbon::nova(2, "Oculta").visivel(false), AbaRibbon::nova(3, "Desabilitada").habilitada(false)];
        let grupos = [
            GrupoRibbon::novo("area", "Área", vec![AcaoRibbon::nova("colar", "Colar", Some(Msg::Ribbon("colar"))), AcaoRibbon::nova("copiar", "Copiar", Some(Msg::Ribbon("copiar"))).ativa(true)]),
            GrupoRibbon::novo("fonte", "Fonte", vec![AcaoRibbon::nova("n", "N", Some(Msg::Ribbon("negrito"))), AcaoRibbon::nova("i", "I", None)]),
        ];
        let menu_aberto: Option<Element<'_, Msg>> = self.menu.aberto.then(|| ribbon::menu(&itens_menu(), &self.menu).map(Msg::Menu));
        cartoes.push(cartao(
            "ribbon",
            "Abas (uma oculta, uma desabilitada), ferramentas por grupo e menu com submenu.",
            estreito,
            column![
                ribbon::abas(&abas_ribbon, &self.aba, Msg::Aba),
                container(ribbon::ferramentas(&grupos)).height(56.0),
                botao(if self.menu.aberto { "Fechar menu" } else { "Abrir menu" }, Msg::AbrirMenu),
            ]
            .push_maybe(menu_aberto)
            .spacing(6),
        ));

        let filtrados = self.dropdown.indices_filtrados(&OPCOES_DD);
        let mut lista_dd = column![].spacing(2);
        if self.dropdown.aberto {
            for i in filtrados {
                let destaque = self.dropdown.destaque == Some(i);
                lista_dd = lista_dd.push(button(text(OPCOES_DD[i]).size(12)).width(Length::FILL).padding([3u16, 8]).style(estilo_botao_pequeno(destaque)).on_press(Msg::Dropdown(EventoDropdown::Escolher(i))));
            }
        }
        cartoes.push(cartao(
            "dropdown (máquina de estados)",
            "O estado só (sem visual próprio): Alternar, ↑/↓, Confirmar.",
            estreito,
            column![
                row![
                    botao(self.escolhido_dd.map_or("Escolha…", |i| OPCOES_DD[i]), Msg::Dropdown(EventoDropdown::Alternar)),
                    botao("↑", Msg::Dropdown(EventoDropdown::Cima)),
                    botao("↓", Msg::Dropdown(EventoDropdown::Baixo)),
                    botao("Confirmar", Msg::Dropdown(EventoDropdown::Confirmar)),
                ]
                .spacing(6),
                lista_dd,
            ]
            .spacing(6),
        ));

        cartoes.push(cartao(
            "campo_numero, alinhamento, efeitos_imagem",
            "Setas do campo passam de 0,5 em 0,5 entre 0 e 100.",
            estreito,
            column![
                campo_numero(&self.numero, "Valor", 120.0, Limites { passo: 0.5, min: Some(0.0), max: Some(100.0) }, feather_ui::campo_numero::ler(&self.numero).is_none(), Msg::Numero),
                alinhamento::seletor_horizontal(self.alinhamento, true, true, Msg::Alinhar),
                efeitos_imagem::painel(self.efeito, Msg::Efeito, Msg::Efeito("original")),
            ]
            .spacing(10),
        ));

        cartoes.push(cartao(
            "componentes",
            "botao_grande, botao_icone (com dica), lista_suspensa e lista_pesquisavel (combo_box).",
            estreito,
            column![
                row![
                    botao_grande(icone("pencil", 24.0), "Grande", Some(Msg::Grande), false),
                    botao_grande(icone("highlight", 24.0), "Ativo", Some(Msg::Grande), true),
                    botao_icone(icone("search", 18.0), "Buscar", Some(Msg::Grande), false),
                    botao_icone(icone("settings", 18.0), "Desabilitado", None, false),
                ]
                .spacing(6)
                .align(Align::Center),
                row![lista_suspensa("Formato", &FORMATOS, self.lista, Msg::Lista, 120.0), lista_pesquisavel(&self.pesquisaveis, "Pesquise uma fonte…", Msg::Pesquisa, 180.0)].spacing(8),
            ]
            .spacing(8),
        ));

        cartoes.push(cartao(
            "dialogo, ajuda e créditos",
            "Modal com véu (Esc fecha) e a ajuda pesquisável (F1).",
            estreito,
            column![row![botao("Abrir diálogo", Msg::AbrirDialogo), botao("Abrir ajuda (F1)", Msg::AbrirAjuda)].spacing(6), creditos::colaboradores()].spacing(12),
        ));

        let secoes = ["Aparência", "Editor", "Atalhos", "Sobre"];
        let menu_cfg = configuracoes::menu("Configurações", secoes.iter().enumerate().map(|(i, s)| (s.to_string(), i == self.config, Msg::Config(i))));
        let corpo_cfg = column![
            configuracoes::linha("Tema escuro", "Troca as cores do feather-ui na hora.", toggler("", self.escuro).on_toggle(Msg::Escuro)),
            configuracoes::linha("Formato padrão", "Uma lista suspensa como controle.", lista_suspensa("Formato", &FORMATOS, self.lista, Msg::Lista, 120.0)),
        ];
        cartoes.push(cartao("configuracoes", "menu + tela + linha.", largo, container(configuracoes::tela(menu_cfg, secoes[self.config], corpo_cfg.into(), None)).height(260.0).clip(true)));

        let mut camada = tinta::Camada::nova(self.tracos.clone(), Transformacao::default(), Msg::Tinta);
        camada.ferramenta = self.tinta.ferramenta_ativa();
        camada.pena = self.tinta.pena().map(|p| (cor_rgba(p.cor), p.espessura));
        camada.fonte_pressao = Some(self.pressao.clone());
        camada.gesto_externo = self.gesto_caneta.clone();
        let faixa = row(tinta::grupos_faixa(&self.tinta).into_iter().map(|e| e.map(Msg::TintaFaixa))).height(100.0);
        let quadro = stack![container(text(if self.tinta.ativo { "desenhe aqui" } else { "escolha uma caneta acima" }).size(12).color(COR_TEXTO_MUTED())).padding(8).width(Length::FILL).height(Length::FILL).style(|_| Style::default().background(Color::WHITE).border(Border::rounded(6.0).color(COR_BORDA(), 1.0))), tinta::camada(camada)].height(220.0);
        cartoes.push(cartao(
            "tinta (caneta / mouse)",
            "Faixa Desenhar + camada. Mouse, toque ou caneta (pressão pela mesa no X11). Esc para de desenhar.",
            largo,
            column![container(faixa).clip(true)].push_maybe(tinta::menu_pena(&self.tinta).map(|m| m.map(Msg::TintaFaixa))).push(quadro).spacing(6),
        ));

        // NOVOS: combo_box, menu_height, tooltip com estilo, line_height e toque.
        cartoes.push(cartao(
            "combo_box e listas longas",
            "Digite para filtrar (sem acento nem maiúscula); ↑/↓, Enter, Esc. O pick_list e a lista pesquisável rolam a partir de 120 px.",
            estreito,
            column![
                combo_box(&self.pesquisaveis, "combo_box: digite uma fonte…", self.combo.as_ref(), Msg::Combo),
                row![
                    pick_list(&self.pesquisaveis[..], self.combo.clone(), Msg::Combo).placeholder("pick_list curto").menu_height(120.0).width(180.0),
                    lista_pesquisavel(&self.pesquisaveis, "lista_pesquisavel…", Msg::Pesquisa, 180.0),
                ]
                .spacing(8),
            ]
            .spacing(8),
        ));
        cartoes.push(cartao(
            "Tooltip estilizado e line_height",
            "A dica usa tema::estilo_dica com padding 10×14. Os parágrafos: altura de linha padrão (1,3) e 1,5.",
            estreito,
            column![
                tooltip(button(text("Dica com estilo").size(12)).padding([5u16, 10]).style(estilo_botao_secundario).on_press(Msg::Dica(TooltipPosition::Bottom)), "Balão com estilo_dica e mais respiro", TooltipPosition::Bottom).style(estilo_dica).padding([10u16, 14]),
                row![
                    text("Padrão: o mesmo parágrafo, para comparar o espaço entre as linhas quebradas.").size(12).width(Length::FILL),
                    text("1,5: o mesmo parágrafo, para comparar o espaço entre as linhas quebradas.").size(12).line_height(1.5).width(Length::FILL),
                ]
                .spacing(12),
            ]
            .spacing(10),
        ));
        cartoes.push(cartao(
            "Toque (Finger*)",
            "Numa tela de toque: cada dedo aparece no log. O primeiro dedo livre vira mouse (botões funcionam); no quadro de tinta, o dedo desenha.",
            estreito,
            text("Sem tela de toque, nada aqui dispara.").size(12).color(COR_TEXTO_MUTED()),
        ));


        let status = container(
            row![
                text(format!("#{}", self.contagem)).size(12).font(Font::MONO).color(COR_TEXTO_MUTED()),
                text(self.ultima.clone()).size(12).font(Font::MONO).width(Length::FILL),
                text(format!("pressão: {}", self.pressao.descricao())).size(11).color(COR_TEXTO_MUTED()),
            ]
            .spacing(10)
            .align(Align::Center),
        )
        .padding([6u16, 14])
        .width(Length::FILL)
        .style(|_| Style::default().background(COR_PAINEL_FAIXA()));

        let corpo = column![status, scrollable(row(cartoes).wrap().spacing(14).padding(14)).id("vitrine")];
        let base = feather_ui::clique_caneta::clique_caneta(
            janela_com_barra("Vitrine do pelicano + feather-ui", corpo.into(), AcoesJanela { arrastar: Msg::Arrastar, minimizar: Msg::Minimizar, maximizar: Msg::Maximizar, fechar: Msg::Fechar }),
            Some(&self.pressao),
        );
        let base = container(base).width(Length::FILL).height(Length::FILL).style(|_| Style::default().background(COR_FUNDO_JANELA()).text_color(COR_TEXTO()));

        if self.ajuda_aberta {
            let corpo = container(self.catalogo.view(&self.ajuda).map(Msg::Ajuda)).height(560.0);
            return Dialogo::new("Ajuda", corpo).icone("help").largura(980.0).altura_maxima(700.0).sem_rolagem().ao_fechar(Msg::Esc).sobre(base);
        }
        if self.dialogo {
            let corpo = column![dlg::secao("Contagens"), dlg::grade_rotulo_valor(vec![("Mensagens".into(), self.contagem.to_string()), ("Traços".into(), self.tracos.len().to_string())], 2), dlg::aviso_vazio("Clique no véu, no x ou aperte Esc para fechar.")].spacing(8);
            return Dialogo::new("Diálogo interno", corpo).icone("chart-bar").subtitulo("feather_ui::dialogo").ao_fechar(Msg::FecharDialogo).secundario("Cancelar", Some(Msg::FecharDialogo)).primario("OK", Some(Msg::FecharDialogo)).sobre(base);
        }
        base.into()
    }
}

fn main() -> Result<(), Error> {
    if let Ok(bytes) = std::fs::read("/usr/share/fonts/truetype/comfortaa/Comfortaa-Regular.ttf") {
        load_font(bytes);
    }
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--medir") {
        medir();
        return Ok(());
    }
    if let Some(i) = args.iter().position(|a| a == "--capturar") {
        let destino = args.get(i + 1).map_or("vitrine.png", String::as_str);
        let app = Vitrine::nova(false);
        // `--capturar saida.png 3000`: mais alta, para ver todos os cartões.
        let altura = args.get(i + 2).and_then(|a| a.parse().ok()).unwrap_or(900.0);
        let png = capture(&app.view(), Size::new(1400.0, altura), 1.0, &app.theme(), &mut Renderer::new(), None);
        std::fs::write(destino, png).map_err(|e| Error::Window(e.to_string()))?;
        println!("{destino}");
        return Ok(());
    }
    caneta::definir_diagnostico(std::env::var_os("VITRINE_CANETA_DIAG").is_some());
    let x11 = caneta::deve_usar_x11(
        caneta::PreferenciaJanela::Automatica,
        !caneta::mesas_conectadas().is_empty(),
        std::env::var("WAYLAND_DISPLAY").ok().as_deref(),
        std::env::var("DISPLAY").ok().as_deref(),
    );
    let settings = Settings { title: "Vitrine".into(), size: Size::new(1400.0, 900.0), min_size: Some(Size::new(720.0, 480.0)), decorations: false, x11, debug: true, ..Default::default() };
    run(settings, || {
        let app = Vitrine::nova(true);
        let fonte = app.pressao.clone();
        let mut tarefas = vec![];
        #[cfg(all(unix, not(target_os = "macos"), not(target_os = "android")))]
        {
            let d = caneta::x11::iniciar(fonte.clone()).unwrap_or_else(|e| format!("sem XInput2: {e}"));
            tarefas.push(Task::done(Msg::Caneta(d)));
            tarefas.push(Task::window_handle(WindowId::MAIN, move |h| {
                let id = match h {
                    raw_window_handle::RawWindowHandle::Xlib(h) => Some(h.window as u32),
                    raw_window_handle::RawWindowHandle::Xcb(h) => Some(h.window.get()),
                    _ => None,
                };
                if let Some(id) = id {
                    fonte.definir_janela_x11(id);
                }
                Msg::Caneta(format!("janela X11: {id:?}"))
            }));
        }
        #[cfg(windows)]
        tarefas.push(Task::window_handle(WindowId::MAIN, move |h| Msg::Caneta(format!("{:?}", caneta::windows::ligar_na_janela(h, fonte)))));
        (app, Task::batch(tarefas))
    })
}

/// `--medir`: rola a vitrine sem janela e mede cada quadro como o `run` faz (evento + quadro).
fn medir() {
    let mut app = Vitrine::nova(false);
    let (tam, tema) = (Size::new(1400.0, 900.0), tema());
    let mut r = Renderer::new();
    let mut st = States::default();
    let mut clip = MemoryClipboard::default();
    let cursor = Some(Point::new(700.0, 300.0));
    let mut tempos = Vec::new();
    for passo in 0..40 {
        let t0 = Instant::now();
        let mut msgs = Vec::new();
        {
            let el = app.view();
            let no = layout(&el, tam, &mut r, &st, &tema);
            let mut janela = Vec::new();
            let mut cx = EventCx { renderer: &mut r, states: &mut st, cursor, messages: &mut msgs, window: &mut janela, redraw: false, clipboard: &mut clip };
            let dy = if passo < 20 { -60.0 } else { 60.0 };
            dispatch(&el, &no, &Event::Wheel { dx: 0.0, dy, modifiers: Modifiers::default() }, &mut cx);
        }
        let t_ev = t0.elapsed();
        for m in msgs {
            let _ = app.update(m);
        }
        let t1 = Instant::now();
        let el = app.view();
        let t_view = t1.elapsed();
        let no = layout(&el, tam, &mut r, &st, &tema);
        let t_layout = t1.elapsed();
        r.record();
        draw(&el, &no, &mut r, &st, &tema, cursor, false);
        let mudou = r.changes();
        let t_rec = t1.elapsed();
        r.begin(tam, std::env::var("VITRINE_ESCALA").ok().and_then(|s| s.parse().ok()).unwrap_or(1.0), tema.background);
        draw(&el, &no, &mut r, &st, &tema, cursor, false);
        let total = t0.elapsed();
        if passo < 3 || passo % 10 == 0 {
            eprintln!("passo {passo}: evento {t_ev:?} view {t_view:?} layout {t_layout:?} gravar {t_rec:?} total {total:?} ({})", match mudou { Changes::All => "tudo".into(), Changes::None => "nada".into(), Changes::Regions(v) => format!("{} regiões", v.len()) });
        }
        tempos.push(total.as_secs_f64() * 1000.0);
    }
    tempos.sort_by(|a, b| a.total_cmp(b));
    eprintln!("mediana {:.1} ms, p90 {:.1} ms", tempos[tempos.len() / 2], tempos[tempos.len() * 9 / 10]);
}
