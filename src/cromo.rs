//! Barra de edição e superfícies de navegação comuns aos aplicativos.
//! Eventos e operações continuam pertencendo a cada aplicativo.
use pelicano::{
    button, column, container, horizontal_space, mouse_area, row, svg, text, tooltip, Align, Border, Color, Element,
    Length, Space, Status, Style, SvgView, TooltipPosition,
};

pub const BARRA_ALTURA: f32 = 42.0;

/// Moldura externa comum. A margem reserva o traço para que o conteúdo
/// opaco e os painéis sobrepostos não cubram a borda da janela.
pub fn moldura_janela<'a, M: 'a>(conteudo: Element<'a, M>) -> Element<'a, M> {
    container(conteudo)
        .padding(1)
        .width(Length::FILL)
        .height(Length::FILL)
        .style(|_| Style::default().background(COR_FUNDO_JANELA()).border(Border::default().color(COR_BORDA(), 1.0)))
        .into()
}

/// Espessura da borda que redimensiona a janela sem moldura do sistema.
pub const BORDA_REDIMENSIONAR: f32 = 4.0;

/// Raio dos cantos da janela com barra própria (fora de maximizada).
pub const RAIO_JANELA: f32 = 14.0;

/// [`moldura_janela`] para janela sem moldura do sistema (barra própria): uma faixa de
/// [`BORDA_REDIMENSIONAR`] px em volta, com o cursor de redimensionar, que pede
/// `ao_redimensionar(direção)` — o app responde com `WindowCmd::DragResize(direção)`.
/// Sem ela, a janela com barra própria só mudava de tamanho maximizando.
pub fn moldura_redimensionavel<'a, M: Clone + 'a>(
    conteudo: Element<'a, M>,
    ao_redimensionar: impl Fn(pelicano::ResizeDirection) -> M,
) -> Element<'a, M> {
    bordas_redimensionar(moldura_janela(conteudo), false, ao_redimensionar)
}

/// [`moldura_redimensionavel`] para a janela aberta com `transparent: true`, sabendo se
/// ela está maximizada (`pelicano::Event::Maximized`). Fora de maximizada: cantos
/// arredondados ([`RAIO_JANELA`]) e a faixa de redimensionar transparente — pintada com o
/// fundo e com a borda reta de 1 px, parecia uma moldura estranha em volta da janela.
/// Maximizada: sem faixa nem cantos. No Windows o pelicano não faz janela transparente:
/// lá fica a moldura reta de sempre.
pub fn moldura_redimensionavel_com<'a, M: Clone + 'a>(
    conteudo: Element<'a, M>,
    maximizada: bool,
    ao_redimensionar: impl Fn(pelicano::ResizeDirection) -> M,
) -> Element<'a, M> {
    if maximizada {
        return container(conteudo).width(Length::FILL).height(Length::FILL)
            .style(|_| Style::default().background(COR_FUNDO_JANELA())).into();
    }
    if cfg!(windows) {
        return moldura_redimensionavel(conteudo, ao_redimensionar);
    }
    // `clip` corta o conteúdo (a barra de título) no raio dos cantos.
    let janela = container(conteudo).padding(1).clip(true).width(Length::FILL).height(Length::FILL)
        .style(|_| Style::default().background(COR_FUNDO_JANELA())
            .border(Border::rounded(RAIO_JANELA).color(COR_BORDA(), 1.0)))
        .into();
    bordas_redimensionar(janela, true, ao_redimensionar)
}

/// A faixa de [`BORDA_REDIMENSIONAR`] px em volta de `janela`, com o cursor de cada lado.
fn bordas_redimensionar<'a, M: Clone + 'a>(
    janela: Element<'a, M>,
    transparente: bool,
    ao_redimensionar: impl Fn(pelicano::ResizeDirection) -> M,
) -> Element<'a, M> {
    use pelicano::{CursorIcon as C, ResizeDirection as D};
    let b = BORDA_REDIMENSIONAR;
    let borda = |w: Length, h: Length, d: D, c: C| -> Element<'a, M> {
        mouse_area(container(Space::new(w, h)).style(move |_| if transparente { Style::default() } else { Style::default().background(COR_FUNDO_JANELA()) }))
            .on_press(ao_redimensionar(d))
            .cursor_icon(c)
            .into()
    };
    let fixo = Length::Fixed(b);
    column![
        row![
            borda(fixo, fixo, D::NorthWest, C::NwResize),
            borda(Length::FILL, fixo, D::North, C::NResize),
            borda(fixo, fixo, D::NorthEast, C::NeResize),
        ],
        row![
            borda(fixo, Length::FILL, D::West, C::WResize),
            janela,
            borda(fixo, Length::FILL, D::East, C::EResize),
        ]
        .height(Length::FILL),
        row![
            borda(fixo, fixo, D::SouthWest, C::SwResize),
            borda(Length::FILL, fixo, D::South, C::SResize),
            borda(fixo, fixo, D::SouthEast, C::SeResize),
        ],
    ]
    .width(Length::FILL)
    .height(Length::FILL)
    .into()
}

/// Os eventos continuam sendo do programa hospedeiro.
pub struct AcoesJanela<M> {
    pub arrastar: M,
    pub minimizar: M,
    pub maximizar: M,
    pub fechar: M,
}

use crate::{icones::icone, marca, tema::*};

pub struct Acao<M> {
    pub icone: &'static str,
    pub dica: &'static str,
    pub mensagem: Option<M>,
}

/// Pena vetorial do produto ativo — cor de fábrica ou cor personalizada.
pub fn emblema<'a, M: 'a>() -> Element<'a, M> {
    marca::icone_atual(27.).into()
}

/// Ação compacta com descrição acessível ao passar o ponteiro.
pub fn acao<'a, M: Clone + 'static>(nome: &str, dica: &str, mensagem: Option<M>) -> Element<'a, M> {
    acao_icone(icone(nome, 19.), dica, mensagem)
}

/// Mesmo botão da barra, aceitando ícones próprios do aplicativo (Slide/CAD).
pub fn acao_icone<'a, M: Clone + 'static>(
    desenho: SvgView,
    dica: &str,
    mensagem: Option<M>,
) -> Element<'a, M> {
    let botao = button(desenho).padding([7, 9]).style(estilo_botao_pequeno(false)).on_press_maybe(mensagem);
    Element::from(tooltip(botao, dica.to_owned(), TooltipPosition::Bottom).padding([4, 8]).style(crate::tema::estilo_dica)).access_label(dica.to_owned())
}

/// Linha superior do editor, compatível com mensagens próprias de cada app.
/// Controles da janela (minimizar/arrastar) ficam na aplicação hospedeira.
pub fn barra_editor<'a, M: Clone + 'static>(
    arquivo: &str,
    alterado: bool,
    abrir_arquivos: Option<M>,
    acoes: impl IntoIterator<Item = Acao<M>>,
    abrir_configuracoes: M,
) -> Element<'a, M> {
    // `None`: o app já tem outra entrada para o menu de arquivos (o FeatherSheet
    // tem o botão Menu na faixa) e o botão "Arquivos" seria repetido.
    let mut itens = row![emblema()].spacing(2).align(Align::Center);
    if let Some(abrir) = abrir_arquivos {
        itens = itens.push(button(text(crate::idioma::t("Arquivos")).size(12.5)).padding([7, 10])
            .style(estilo_botao_pequeno(false)).on_press(abrir));
    }
    itens = itens.push(Space::new(6, 0));
    for item in acoes {
        itens = itens.push(acao(item.icone, item.dica, item.mensagem));
    }
    itens = itens.push(Space::new(10, 0))
        .push(row![
            text(arquivo.to_owned()).size(13),
            text(if alterado { crate::idioma::t("• não salvo") } else { crate::idioma::t("salvo") })
                .size(11.5).color(COR_TEXTO_MUTED()),
        ].spacing(8).align(Align::Center))
        .push(horizontal_space())
        .push(acao("settings", crate::idioma::t("Configurações"), Some(abrir_configuracoes)));
    column![
        container(itens).padding([0, 12]).align_y(Align::Center).width(Length::FILL)
            .height(BARRA_ALTURA).style(estilo_container_topo),
        linha(),
    ].into()
}

fn linha<'a, M: 'a>() -> Element<'a, M> {
    container(Space::new(Length::FILL, 1)).width(Length::FILL).style(estilo_linha).into()
}

/// Ícones de janela com os mesmos traços Tabler usados pelo FeatherSlide.
fn icone_janela(caminho: &str) -> SvgView {
    let data = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"#5f6368\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\">{caminho}</svg>"
    );
    svg(crate::icones::carregar(data)).color(COR_TEXTO_MUTED()).width(16).height(16)
}

/// Copiado do FeatherSlide: largura 48, altura 42, hover neutro e fechamento vermelho.
pub fn controles_janela<'a, M: Clone + 'static>(
    acoes: AcoesJanela<M>,
) -> Element<'a, M> {
    let botao = |desenho: SvgView, msg: M, fechar: bool, nome: &'static str| -> Element<'a, M> {
        let b = button(container(desenho).center())
            .width(48)
            .height(BARRA_ALTURA)
            .padding(0)
            .style(move |_, status| {
                let hover = matches!(status, Status::Hovered | Status::Pressed);
                let fundo = if !hover { Color::TRANSPARENT } else if fechar { Color::rgb8(0xe8, 0x11, 0x23) } else { COR_HOVER() };
                Style::default().background(fundo).text_color(COR_TEXTO())
            })
            .on_press(msg);
        Element::from(b).access_label(crate::idioma::t(nome))
    };
    container(row![
        botao(icone_janela("<path d=\"M5 12h14\"/>"), acoes.minimizar, false, "Minimizar"),
        botao(icone_janela("<rect x=\"5\" y=\"5\" width=\"14\" height=\"14\" rx=\"1\"/>"), acoes.maximizar, false, "Maximizar"),
        botao(icone_janela("<path d=\"M18 6L6 18M6 6l12 12\"/>"), acoes.fechar, true, "Fechar"),
    ]).style(estilo_container_topo).into()
}

/// Aplica a barra personalizada ao corpo de qualquer janela auxiliar (Configurações, F1, F12).
pub fn janela_com_barra<'a, M: Clone + 'static>(
    titulo: &str,
    conteudo: Element<'a, M>,
    acoes: AcoesJanela<M>,
) -> Element<'a, M> {
    let nome = container(row![
        text(titulo.to_owned()).size(13),
        horizontal_space(),
    ].align(Align::Center))
        .padding([0, 14]).align_y(Align::Center)
        .width(Length::FILL).height(BARRA_ALTURA)
        .style(estilo_container_topo);
    let arrastar = acoes.arrastar.clone();
    let barra = row![
        mouse_area(nome).on_press(arrastar),
        controles_janela(acoes),
    ].align(Align::Center);
    column![barra, linha(), container(conteudo).width(Length::FILL).height(Length::FILL)].into()
}

/// Combina uma barra de edição do app com os controles nativos desenhados.
pub fn barra_integrada<'a, M: Clone + 'static>(
    barra: Element<'a, M>,
    acoes: AcoesJanela<M>,
) -> Element<'a, M> {
    let arrastar = acoes.arrastar.clone();
    column![
        row![
            mouse_area(barra).on_press(arrastar),
            controles_janela(acoes),
        ].align(Align::Center),
        linha(),
    ].into()
}

/// Menu de arquivos da suíte, com as ações disponíveis do aplicativo.
pub fn menu_arquivos<'a, M: Clone + 'a>(
    aplicativo: &str,
    opcoes: impl IntoIterator<Item = (&'static str, &'static str, M)>,
) -> Element<'a, M> {
    let mut itens = column![
        text(aplicativo.to_owned()).size(18),
        text(crate::idioma::t("Arquivos e ferramentas")).size(12).color(COR_TEXTO_MUTED()),
        Space::new(0, 8),
    ].spacing(6);
    for (icone_nome, rotulo, mensagem) in opcoes {
        itens = itens.push(
            button(row![icone(icone_nome, 17.), text(rotulo).size(13)]
                .spacing(10).align(Align::Center))
                .width(Length::FILL).padding([9, 12])
                .style(estilo_botao_pequeno(false)).on_press(mensagem),
        );
    }
    container(itens.padding(14)).width(280)
        .style(estilo_painel_miniaturas).into()
}

/// Cabeçalho usado por Ajuda (F1), Monitor (F12) e outros painéis utilitários.
pub fn painel_utilitario<'a, M: Clone + 'a>(
    titulo: &str,
    conteudo: Element<'a, M>,
    fechar: M,
) -> Element<'a, M> {
    container(column![
        row![
            text(titulo.to_owned()).size(20),
            horizontal_space(),
            button(text(crate::idioma::t("Fechar")).size(12)).padding([6, 12])
                .style(estilo_botao_secundario).on_press(fechar),
        ].align(Align::Center),
        conteudo,
    ].spacing(14)).padding(22).width(Length::FILL)
        .style(estilo_painel_miniaturas).into()
}

#[cfg(test)]
mod testes {
    use super::*;
    use pelicano::column;

    #[test]
    fn barras_montam_e_fazem_layout() {
        let acoes = || AcoesJanela { arrastar: 0u8, minimizar: 1, maximizar: 2, fechar: 3 };
        let barra = barra_editor("a.txt", true, Some(4u8), [Acao { icone: "x", dica: "Fechar", mensagem: None }], 5);
        let tela = column![barra_integrada(barra, acoes()), janela_com_barra("Ajuda", menu_arquivos("Docs", [("x", "Sair", 6)]), acoes())];
        let tela = moldura_janela(column![tela, painel_utilitario("F12", text("ok").into(), 7)].into());
        let no = pelicano::layout(&tela, pelicano::Size::new(900.0, 700.0), &mut pelicano::Renderer::new(), &Default::default(), &crate::tema::tema());
        assert_eq!(no.rect.width, 900.0);
    }
}
