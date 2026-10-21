//! Componentes genéricos reutilizáveis da faixa do FeatherWorks.
//! A mensagem e os eventos continuam sendo propriedade do aplicativo:
//! o crate de UI nunca importa nenhum motor de documentos, planilhas ou slides.

use pelicano::{button, column, combo_box, container, pick_list, row, text, tooltip, vertical_rule, Align, Element, Length, SvgView, TooltipPosition};

use crate::tema::{estilo_botao_grande, estilo_botao_pequeno, COR_BORDA, COR_TEXTO_MUTED};

/// Altura máxima dos menus suspensos; passando dela, eles rolam.
pub const ALTURA_MAX_DROPDOWN: f32 = 256.0;

/// Botão de ferramenta com ícone acima do rótulo (faixa/Ribbon).
pub fn botao_grande<'a, M: Clone + 'a>(
    icone: SvgView,
    rotulo: &str,
    mensagem: Option<M>,
    ativo: bool,
) -> Element<'a, M> {
    button(
        column![icone, text(rotulo.to_owned()).size(11).no_wrap()]
            .align(Align::Center)
            .spacing(3),
    )
    .padding([4u16, 8])
    .style(estilo_botao_grande(ativo))
    .on_press_maybe(mensagem)
    .into()
}

/// Botão compacto de ferramenta, com ícone à esquerda do rótulo.
pub fn botao_pequeno<'a, M: Clone + 'a>(
    icone: SvgView,
    rotulo: &str,
    mensagem: Option<M>,
    ativo: bool,
) -> Element<'a, M> {
    button(
        row![icone, text(rotulo.to_owned()).size(11).no_wrap()]
            .spacing(5)
            .align(Align::Center),
    )
    .padding([2u16, 6])
    .style(estilo_botao_pequeno(ativo))
    .on_press_maybe(mensagem)
    .into()
}

/// Botão só com o ícone: o nome da ação fica na dica, sem texto duplicado
/// dentro da faixa. Reutilizável no Sheet, Slide e Docs.
pub fn botao_icone<'a, M: Clone + 'a>(
    icone: SvgView,
    dica: &str,
    mensagem: Option<M>,
    ativo: bool,
) -> Element<'a, M> {
    let controle = button(icone)
        .padding([4u16, 6])
        .style(estilo_botao_pequeno(ativo))
        .on_press_maybe(mensagem);
    Element::from(tooltip(controle, dica.to_owned(), TooltipPosition::Bottom).padding([4, 6]).style(crate::tema::estilo_dica)).access_label(dica.to_owned())
}

/// Dropdown de aparência compartilhada. A mensagem pertence ao aplicativo.
pub fn lista_suspensa<'a, M: Clone + 'a>(
    rotulo: &'static str,
    opcoes: &'static [&'static str],
    selecionado: Option<&'static str>,
    escolher: impl Fn(&'static str) -> M + 'a,
    largura: f32,
) -> Element<'a, M> {
    lista_tipada(rotulo, opcoes, selecionado, escolher, largura)
}

/// Lista tipada para apps que usam valores numéricos, enums ou &str dinâmico.
pub fn lista_tipada<'a, M, T>(
    rotulo: &'static str,
    opcoes: &'a [T],
    selecionado: Option<T>,
    escolher: impl Fn(T) -> M + 'a,
    largura: f32,
) -> Element<'a, M>
where
    M: Clone + 'a,
    T: Clone + PartialEq + ToString + 'a,
{
    pick_list(opcoes, selecionado, escolher)
        .placeholder(rotulo)
        .size(11)
        .width(largura)
        .menu_height(ALTURA_MAX_DROPDOWN)
        .into()
}

/// Dropdown das opções do app que filtra enquanto se digita.
pub fn lista_pesquisavel<'a, M: Clone + 'a>(
    opcoes: &'a [String],
    dica: &'static str,
    escolher: impl Fn(String) -> M + 'a,
    largura: f32,
) -> Element<'a, M> {
    combo_box(opcoes, dica, None, escolher)
        .size(11)
        .padding([4u16, 8])
        .width(largura)
        .menu_height(ALTURA_MAX_DROPDOWN)
        .into()
}

/// Grupo vertical da faixa: conteúdo, legenda e linha separadora.
pub fn grupo_faixa<'a, M: 'a>(
    itens: Element<'a, M>,
    rotulo: &'static str,
) -> Element<'a, M> {
    row![
        container(
            column![
                container(itens).height(Length::FILL).align_y(Align::Center).clip(true),
                text(rotulo).size(11).no_wrap().color(COR_TEXTO_MUTED()),
            ]
            .align(Align::Center)
            .spacing(2),
        )
        .padding([4u16, 12]),
        vertical_rule(1.0).color(COR_BORDA()),
    ]
    .into()
}

/// Alturas explícitas da Ribbon compacta. A legenda não disputa espaço com
/// os controles (evita que fique escondida pela barra de rolagem horizontal).
pub const ALTURA_CONTEUDO_GRUPO_COMPACTO: f32 = 68.0;
pub const ALTURA_GRUPO_COMPACTO: f32 = 96.0;

/// Variação para faixas de duas linhas, como a de Fonte no FeatherSheet.
/// Deixa a legenda sempre na parte inferior do grupo, dentro da Ribbon,
/// sem alterar o estilo dos demais aplicativos que usam `grupo_faixa`.
pub fn grupo_faixa_compacto<'a, M: 'a>(
    itens: Element<'a, M>,
    rotulo: &'static str,
) -> Element<'a, M> {
    row![
        container(
            column![
                container(itens)
                    .height(ALTURA_CONTEUDO_GRUPO_COMPACTO)
                    .align_y(Align::Center)
                    .clip(true),
                text(rotulo).size(11).no_wrap().color(COR_TEXTO_MUTED()),
            ]
            .align(Align::Center)
            .spacing(3),
        )
        .padding([3u16, 10])
        .height(ALTURA_GRUPO_COMPACTO),
        vertical_rule(1.0).color(COR_BORDA()),
    ]
    .height(ALTURA_GRUPO_COMPACTO)
    .into()
}

#[cfg(test)]
mod testes {
    use super::{grupo_faixa_compacto, ALTURA_CONTEUDO_GRUPO_COMPACTO, ALTURA_GRUPO_COMPACTO};
    use crate::tema::{acento_hex, definir_tema, ACENTOS};
    use pelicano::{text, Element};

    #[test]
    fn grupo_compacto_reserva_legenda_antes_da_rolagem() {
        // 68 de corpo + 3 de intervalo + 14 de legenda + 6 de padding
        // cabem em 96; a Ribbon reserva 100 para seus grupos.
        const _: () = assert!(ALTURA_CONTEUDO_GRUPO_COMPACTO + 3.0 + 14.0 + 6.0 <= ALTURA_GRUPO_COMPACTO);
        const _: () = assert!(ALTURA_GRUPO_COMPACTO <= 100.0);
        let _grupo: Element<'_, ()> =
            grupo_faixa_compacto(text("controles").into(), "Fonte");
    }

    #[test]
    fn paletas_e_acento_fora_do_intervalo() {
        // O tema usa estado global para refletir as Configurações em toda a
        // interface. Testar no mesmo caso evita duas threads concorrentes
        // alterando o acento enquanto a outra lê o resultado.
        for (indice, &(_, claro, escuro)) in ACENTOS.iter().enumerate() {
            definir_tema(false, indice);
            assert_eq!(acento_hex(), claro);
            definir_tema(true, indice);
            assert_eq!(acento_hex(), escuro);
        }
        definir_tema(false, usize::MAX);
        assert_eq!(acento_hex(), ACENTOS[ACENTOS.len() - 1].1);
        definir_tema(false, 0);
    }
}
