//! Estilos visuais e cores que reproduzem exatamente o design da suíte FeatherWorks.

use pelicano::{Border, Color, Point, Shadow, Status, Style, Theme};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

// Paleta da suíte, clara ou escura, com a cor de destaque escolhida nas Configurações.
// Cada cor é uma função (nome em maiúsculas, para manter o jeito de usar de antes) porque
// muda com o tema em tempo de execução.

static ESCURO: AtomicBool = AtomicBool::new(false);
static ACENTO: AtomicUsize = AtomicUsize::new(0);

/// As cores de destaque: (nome, claro, escuro).
pub const ACENTOS: [(&str, &str, &str); 6] = [
    ("Petróleo", "#2b7a78", "#4fb3ad"),
    ("Azul", "#0b57d0", "#8ab4f8"),
    ("Verde", "#188038", "#81c995"),
    ("Laranja", "#e8710a", "#fcad70"),
    ("Vermelho", "#d93025", "#f28b82"),
    ("Roxo", "#8e24aa", "#d7aefb"),
];

/// Resolve a preferência automática pelo tema do sistema (`pelicano::system_dark`).
pub fn escuro_resolvido(automatico: bool, escuro_preferido: bool) -> bool {
    if automatico {
        pelicano::system_dark()
    } else {
        escuro_preferido
    }
}

pub fn definir_tema(escuro: bool, acento: usize) {
    ESCURO.store(escuro, Ordering::Relaxed);
    ACENTO.store(acento.min(ACENTOS.len() - 1), Ordering::Relaxed);
}

pub fn escuro() -> bool {
    ESCURO.load(Ordering::Relaxed)
}

/// O tema do pelicano com as cores da suíte: devolva no `App::theme`. Os controles que não
/// recebem estilo (caixas, chaves, menus, abas) pegam o acento e as superfícies daqui.
pub fn tema() -> Theme {
    Theme {
        background: COR_FUNDO_JANELA(),
        surface: COR_PAINEL(),
        text: COR_TEXTO(),
        muted: COR_TEXTO_MUTED(),
        primary: COR_AZUL_ACENTO(),
        on_primary: crate::acessibilidade::texto_sobre(COR_AZUL_ACENTO()),
        border: COR_BORDA(),
        danger: COR_ERRO(),
    }
}

fn hex(s: &str) -> Color {
    let v = u32::from_str_radix(s.trim_start_matches('#'), 16).unwrap_or(0);
    Color::rgb8((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

fn par(claro: &str, escuro_: &str) -> Color {
    hex(if escuro() { escuro_ } else { claro })
}

fn mistura(a: Color, b: Color, t: f32) -> Color {
    Color { r: a.r + (b.r - a.r) * t, g: a.g + (b.g - a.g) * t, b: a.b + (b.b - a.b) * t, a: 1.0 }
}

/// A cor de destaque em texto `#rrggbb` (para os ícones SVG).
pub fn acento_hex() -> &'static str {
    let (_, c, e) = ACENTOS[ACENTO.load(Ordering::Relaxed)];
    if escuro() { e } else { c }
}

/// Traduz as cores fixas dos ícones (`#5f6368`…) para as do tema atual.
pub fn cor_do_icone(cor: &str) -> String {
    let c = match cor {
        "#1a73e8" | "#0b57d0" => acento_hex().to_string(),
        "#5f6368" => if escuro() { "#9aa0a6".into() } else { cor.into() },
        "#202124" => if escuro() { "#e8eaed".into() } else { cor.into() },
        "#bdc1c6" => if escuro() { "#5f6368".into() } else { cor.into() },
        outra => outra.to_string(),
    };
    c
}

#[allow(non_snake_case)]
pub fn COR_FUNDO_JANELA() -> Color { par("#f1f3f4", "#181a1d") }
#[allow(non_snake_case)]
pub fn COR_PAINEL() -> Color { par("#ffffff", "#2a2d32") }
#[allow(non_snake_case)]
pub fn COR_PAINEL_FAIXA() -> Color { par("#f8f9fa", "#2d3035") }
#[allow(non_snake_case)]
pub fn COR_BORDA() -> Color { par("#dadce0", "#3d4147") }
#[allow(non_snake_case)]
pub fn COR_BORDA_SUAVE() -> Color { par("#e8eaed", "#33363b") }
/// Trilho e polegar da barra de rolagem padrão do Iced (`background.weak`/`.strong` dos
/// temas Light/Dark dele), para ficar igual à versão Iced.
#[allow(non_snake_case)]
pub fn COR_TRILHO_ROLAGEM() -> Color { par("#ededed", "#656666") }
#[allow(non_snake_case)]
pub fn COR_POLEGAR_ROLAGEM() -> Color { par("#cbcbcb", "#9a9a9b") }
#[allow(non_snake_case)]
pub fn COR_TEXTO() -> Color { par("#202124", "#e8eaed") }
#[allow(non_snake_case)]
pub fn COR_TEXTO_MUTED() -> Color { par("#5f6368", "#9aa0a6") }
#[allow(non_snake_case)]
pub fn COR_AZUL_ACENTO() -> Color { hex(acento_hex()) }

/// Variação AA do acento apenas quando utilizado como texto na Ribbon.
/// O acento original continua disponível para fundos, SVG e preferências.
#[allow(non_snake_case)]
pub fn COR_ACENTO_TEXTO(fundo: Color) -> Color {
    crate::acessibilidade::texto_acento(COR_AZUL_ACENTO(), fundo)
}

/// Borda de foco nos controles, >= 3:1 sobre a superfície indicada.
#[allow(non_snake_case)]
pub fn COR_FOCO(fundo: Color) -> Color {
    crate::acessibilidade::borda_foco(COR_AZUL_ACENTO(), fundo)
}
#[allow(non_snake_case)]
pub fn COR_AZUL_FUNDO() -> Color { mistura(COR_PAINEL(), COR_AZUL_ACENTO(), if escuro() { 0.22 } else { 0.10 }) }
#[allow(non_snake_case)]
pub fn COR_HOVER() -> Color { par("#dfe3e8", "#3a3e44") }
#[allow(non_snake_case)]
/// A barra de abas da faixa.
pub fn COR_CROMO() -> Color { par("#f4f6f8", "#202327") }
#[allow(non_snake_case)]
/// O corpo da faixa (onde ficam os grupos de botões).
pub fn COR_CROMO_2() -> Color { par("#eaedf1", "#26292e") }
/// Vermelho de erro (texto, borda de campo inválido, ícone de alerta). AA sobre `COR_PAINEL()`.
#[allow(non_snake_case)]
pub fn COR_ERRO() -> Color { par("#d93025", "#f28b82") }
/// Véu atrás de diálogo modal: preto semitransparente, mais denso no tema escuro.
#[allow(non_snake_case)]
pub fn COR_VEU() -> Color { Color { a: if escuro() { 0.60 } else { 0.45 }, ..Color::BLACK } }
/// Cor da sombra de diálogo (usar com `offset (0, 6)` e `blur_radius 22`).
#[allow(non_snake_case)]
pub fn COR_SOMBRA() -> Color { Color { a: 0.28, ..Color::BLACK } }
#[allow(non_snake_case)]
pub fn COR_HOVER_AZUL() -> Color { mistura(COR_PAINEL(), COR_AZUL_ACENTO(), if escuro() { 0.32 } else { 0.20 }) }
#[allow(non_snake_case)]
pub fn COR_MESA() -> Color { par("#e8eaed", "#2b2e33") }

/// Botão Pílula "Página inicial" no topo (arredondado com borda fina)
pub fn estilo_pilha_inicio(_: &Theme, status: Status) -> Style {
    let bg = match status {
        Status::Hovered => COR_HOVER(),
        Status::Pressed => COR_AZUL_FUNDO(),
        _ => COR_PAINEL(),
    };
    Style::default().background(bg).text_color(COR_TEXTO()).border(Border::rounded(16.0).color(COR_BORDA(), 1.0))
}

/// Botão Primário Azul ("Nova apresentação")
pub fn estilo_botao_primario(_: &Theme, status: Status) -> Style {
    let bg = match status {
        Status::Hovered => Color { r: 0.090, g: 0.396, b: 0.800, a: 1.0 },
        Status::Pressed => Color { r: 0.082, g: 0.345, b: 0.690, a: 1.0 },
        Status::Disabled => Color { r: 0.741, g: 0.757, b: 0.776, a: 1.0 },
        _ => COR_AZUL_ACENTO(),
    };
    Style::default().background(bg).text_color(crate::acessibilidade::texto_sobre(bg)).border(Border::rounded(6.0))
}

/// Botão Secundário Branco com Borda ("Abrir…")
pub fn estilo_botao_secundario(_: &Theme, status: Status) -> Style {
    let bg = match status {
        Status::Hovered => COR_HOVER(),
        Status::Pressed => COR_AZUL_FUNDO(),
        _ => COR_PAINEL(),
    };
    Style::default().background(bg).text_color(COR_TEXTO()).border(Border::rounded(6.0).color(COR_BORDA(), 1.0))
}

/// Botão Redondo para ações no topo (Configurações, Ajuda)
pub fn estilo_botao_circular(_: &Theme, status: Status) -> Style {
    let s = Style::default().text_color(COR_TEXTO_MUTED()).border(Border::rounded(17.0));
    match status {
        Status::Hovered => s.background(COR_HOVER()),
        Status::Pressed => s.background(COR_AZUL_FUNDO()),
        _ => s,
    }
}

/// Aba da Faixa de Opções (Ribbon Tab)
pub fn estilo_aba_faixa(ativa: bool) -> impl Fn(&Theme, Status) -> Style {
    move |_, status| {
        if ativa {
            return Style::default().text_color(COR_ACENTO_TEXTO(COR_CROMO())).border(Border::rounded(3.0).color(COR_AZUL_ACENTO(), 2.0));
        }
        let s = Style::default().border(Border::rounded(3.0));
        match status {
            Status::Hovered => s.background(COR_HOVER()).text_color(COR_TEXTO()),
            _ => s.text_color(COR_TEXTO_MUTED()),
        }
    }
}

/// Dimensões do menu TS (.menu-panel): painel compacto, sem largura de sidebar.
pub const LARGURA_MENU: f32 = 272.0;
pub const LARGURA_SUBMENU: f32 = 236.0;

/// Painel suspenso nativo equivalente a .menu-panel/.menu-sub do TypeScript.
pub fn estilo_popup_menu(_: &Theme) -> Style {
    Style::default()
        .background(COR_PAINEL())
        .border(Border::rounded(8.0).color(COR_BORDA_SUAVE(), 1.0))
        .shadow(Shadow { color: Color { a: if escuro() { 0.35 } else { 0.18 }, ..Color::BLACK }, offset: Point::new(0.0, 6.0), blur: 18.0 })
}

/// Título da barra de menus: altura 22px, radius 4px, destaque discreto.
pub fn estilo_titulo_menu(ativo: bool) -> impl Fn(&Theme, Status) -> Style {
    move |_, estado| {
        let s = Style::default().text_color(if ativo { COR_TEXTO() } else { COR_TEXTO_MUTED() }).border(Border::rounded(4.0));
        if ativo || matches!(estado, Status::Hovered | Status::Pressed) {
            s.background(if ativo { COR_AZUL_FUNDO() } else { COR_HOVER() })
        } else {
            s
        }
    }
}

/// Item de menu: radius 5px e realce apenas em hover/seleção.
pub fn estilo_item_menu(selecionado: bool, disponivel: bool) -> impl Fn(&Theme, Status) -> Style {
    move |_, estado| {
        let s = Style::default().text_color(if disponivel { COR_TEXTO() } else { COR_TEXTO_MUTED() }).border(Border::rounded(5.0));
        if disponivel && (selecionado || matches!(estado, Status::Hovered | Status::Pressed)) {
            s.background(COR_HOVER())
        } else {
            s
        }
    }
}

/// Fundo e cor do texto de um botão da faixa (grande ou pequeno).
fn botao_faixa(ativo: bool, status: Status, raio: f32, apertado: Color) -> Style {
    let mut s = Style::default().border(Border::rounded(raio));
    if ativo {
        s = s.background(COR_AZUL_FUNDO());
    } else if status == Status::Hovered {
        s = s.background(COR_HOVER());
    } else if status == Status::Pressed {
        s = s.background(apertado);
    }
    s.text_color(if status == Status::Disabled {
        Color { a: 0.4, ..COR_TEXTO() }
    } else if ativo {
        COR_ACENTO_TEXTO(COR_AZUL_FUNDO())
    } else {
        COR_TEXTO()
    })
}

/// Botão Grande na Faixa de Opções (com ícone em cima e rótulo embaixo)
pub fn estilo_botao_grande(ativo: bool) -> impl Fn(&Theme, Status) -> Style {
    move |_, status| botao_faixa(ativo, status, 6.0, COR_AZUL_FUNDO())
}

/// Botão Pequeno Compacto na Faixa (formatação de texto, alinhamento, etc.)
pub fn estilo_botao_pequeno(ativo: bool) -> impl Fn(&Theme, Status) -> Style {
    move |_, status| botao_faixa(ativo, status, 4.0, COR_HOVER_AZUL())
}

/// Barra Superior (Topo)
pub fn estilo_container_topo(_: &Theme) -> Style {
    Style::default().background(COR_PAINEL())
}

/// Painel (cartões e blocos claros, com borda fina)
pub fn estilo_painel_faixa(_: &Theme) -> Style {
    Style::default().background(COR_PAINEL_FAIXA()).border(Border::default().color(COR_BORDA(), 1.0))
}

/// O corpo da faixa: fundo cinza-azulado, sem borda (a linha vem de `linha_vertical`/`linha_horizontal`).
pub fn estilo_corpo_faixa(_: &Theme) -> Style {
    Style::default().background(COR_CROMO_2())
}

/// A barra de abas da faixa.
pub fn estilo_barra_abas(_: &Theme) -> Style {
    Style::default().background(COR_CROMO())
}

/// Uma linha fina (divisória) na cor da borda; o tamanho vem de quem a usa.
pub fn estilo_linha(_: &Theme) -> Style {
    Style::default().background(COR_BORDA())
}

/// O sublinhado da aba ativa.
pub fn estilo_sublinhado_aba(ativa: bool) -> impl Fn(&Theme) -> Style {
    move |_| Style::default().background(if ativa { COR_AZUL_ACENTO() } else { Color::TRANSPARENT })
}

/// Barra Lateral de Miniaturas dos Slides
pub fn estilo_painel_miniaturas(_: &Theme) -> Style {
    Style::default().background(COR_FUNDO_JANELA()).border(Border::default().color(COR_BORDA(), 1.0))
}

/// Painel de Anotações do Apresentador
pub fn estilo_painel_notas(_: &Theme) -> Style {
    Style::default().background(COR_PAINEL()).border(Border::default().color(COR_BORDA(), 1.0))
}

/// Barra de Status no Rodapé
pub fn estilo_rodape(_: &Theme) -> Style {
    Style::default().background(COR_PAINEL()).border(Border::default().color(COR_BORDA(), 1.0))
}

/// Campo de texto (e a busca dos menus pesquisáveis): realce de foco legível nos dois temas.
pub fn estilo_entrada_suspensa(_: &Theme, status: Status) -> Style {
    let focado = status == Status::Focused;
    let sobre = focado || status == Status::Hovered;
    Style::default()
        .background(if sobre { COR_AZUL_FUNDO() } else { COR_PAINEL() })
        .text_color(COR_TEXTO())
        .border(Border::rounded(5.0).color(if focado { COR_FOCO(COR_AZUL_FUNDO()) } else { COR_BORDA() }, if focado { 1.5 } else { 1.0 }))
}

/// O balão de dica (tooltip): painel pequeno com borda fina.
pub fn estilo_dica(_: &Theme) -> Style {
    Style::default().background(COR_PAINEL()).text_color(COR_TEXTO()).border(Border::rounded(4.0).color(COR_BORDA(), 1.0))
}

/// Botão da barra de controles da apresentação: quase invisível até passar o mouse.
pub fn estilo_controle_apresentacao(_: &Theme, status: Status) -> Style {
    let alfa = if matches!(status, Status::Hovered | Status::Pressed) { 0.55 } else { 0.0 };
    Style::default().background(Color::WHITE.alpha(alfa)).text_color(Color::WHITE).border(Border::rounded(14.0))
}

/// Rolagem vertical da suíte (o pelicano já deixa a barra fina ao lado do conteúdo).
pub fn rolagem<'a, M: 'a>(conteudo: impl Into<pelicano::Element<'a, M>>) -> pelicano::Scrollable<'a, M> {
    pelicano::scrollable(conteudo)
}

/// Botão "fechar" de diálogo: ícone `x` (15 px), estilo pequeno, dica "Fechar".
/// Fica no canto direito do cabeçalho; `msg` é a mensagem de fechamento.
pub fn botao_fechar<'a, M: Clone + 'a>(msg: M) -> pelicano::Element<'a, M> {
    let botao = pelicano::button(crate::icones::icone("x", 15.0)).padding([5u16, 7]).style(estilo_botao_pequeno(false)).on_press(msg);
    pelicano::tooltip(botao, crate::idioma::t("Fechar"), pelicano::TooltipPosition::Bottom).style(estilo_dica).into()
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn o_tema_do_pelicano_segue_o_acento_e_o_modo() {
        definir_tema(true, 3);
        let t = tema();
        assert_eq!(t.primary, hex("#fcad70"));
        assert_eq!(t.background, COR_FUNDO_JANELA());
        definir_tema(false, 0);
        assert_eq!(tema().primary, hex("#2b7a78"));
    }
}
