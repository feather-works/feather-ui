//! "O que há de novo" e a decisão das boas-vindas, iguais em todos os apps da suíte.
//! Cada app dá a lista de versões (embutida, nada vem da internet) e as traduções; a tela,
//! a regra de quando abrir e o "trazer para a frente" moram aqui.
use crate::consulta::{self, Ilustracao};
use crate::{icones, tema};
use pelicano::{button, column, row, text, Align, Element, Length, Space, Task, WindowCmd, WindowId};

/// Uma versão publicada e o que ela trouxe.
pub struct Versao {
    pub numero: &'static str,
    pub data: &'static str,
    pub resumo: &'static str,
    /// Figura do cabeçalho da versão.
    pub ilustracao: Ilustracao,
    /// Cartões: ícone, título, descrição curta (até ~84 caracteres).
    pub destaques: &'static [(&'static str, &'static str, &'static str)],
    /// "Também nesta versão": melhorias menores e correções.
    pub tambem: &'static [&'static str],
}

/// Qual janela de boas-vindas abre sozinha.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tela { PrimeirosPassos, Novidades }

/// Na abertura: instalação nova (sem preferências de antes) vê os Primeiros passos; quem
/// atualizou vê as novidades. Já marca como visto (fechar pelo X também conta): o app grava
/// as preferências quando vier `Some`.
pub fn ao_abrir(existia: bool, primeiros_vistos: &mut bool, versao_vista: &mut String, atual: &str) -> Option<Tela> {
    let tela = if !existia && !*primeiros_vistos { Some(Tela::PrimeirosPassos) }
        else if versao_vista != atual { Some(Tela::Novidades) }
        else { None };
    if tela.is_some() {
        *primeiros_vistos = true;
        *versao_vista = atual.to_owned();
    }
    tela
}

/// A janela principal aparece depois da de boas-vindas e ficava por cima, escondendo-a:
/// este passo a traz para a frente quando a principal já está na tela.
// shortcut: espera fixa de 800 ms; trocar por um aviso "janela principal visível" se o pelicano ganhar um.
pub fn trazer_para_frente<M: Send + 'static>(id: WindowId, msg: impl FnOnce(WindowId) -> M + Send + 'static) -> Task<M> {
    let (tx, rx) = pelicano::futures::channel::oneshot::channel::<()>();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(800));
        let _ = tx.send(());
    });
    Task::perform(async move { let _ = rx.await; }, move |_| msg(id))
}

/// O comando que a mensagem de [`trazer_para_frente`] devolve.
pub fn focar<M: Send + 'static>(id: WindowId) -> Task<M> { Task::window_of(id, WindowCmd::Focus) }

/// O que a tela precisa do app: traduzir e as mensagens dos botões.
pub struct Acoes<M> {
    pub escolher: fn(usize) -> M,
    pub ver_no_github: M,
    pub fechar: M,
}

/// A tela "O que há de novo": versões na lateral, cabeçalho, destaques em `colunas` e o resto.
pub fn tela<'a, M: Clone + 'static>(versoes: &'static [Versao], escolhida: usize, instalada: &str, colunas: usize,
    tr: impl Fn(&str) -> String, acoes: Acoes<M>) -> Element<'a, M> {
    let atual = escolhida.min(versoes.len().saturating_sub(1));
    let versao = |n: &str| tr("Versão {}").replacen("{}", n, 1);
    let mut nav = column![consulta::rotulo_grupo(&tr("Versões"))].spacing(2);
    for (i, v) in versoes.iter().enumerate() {
        nav = nav.push(consulta::item_nav(versao(v.numero), if i == 0 { "sparkles" } else { "history" }, i == atual, Some((acoes.escolher)(i))));
    }
    let Some(v) = versoes.get(atual) else { return consulta::janela(consulta::lateral(nav, None), Space::new(0.0, 0.0).into(), None) };
    let tambem = v.tambem.iter().fold(column![].spacing(8), |c, item| {
        c.push(row![icones::icone("check", 15.), text(tr(item)).size(13).line_height(1.4)].spacing(9).align(Align::Start))
    });
    let marca = if v.numero == instalada { tr(" · instalada neste computador") } else { String::new() };
    let conteudo = column![
        consulta::cabecalho(versao(v.numero), format!("{} · {}{marca}", v.data, tr(v.resumo)), Some(consulta::ilustracao(v.ilustracao))),
        consulta::negrito(tr("Destaques"), 18.),
        consulta::grade(v.destaques.iter().map(|(icone, titulo, descricao)| consulta::cartao(icone, tr(titulo), tr(descricao), None)).collect(), colunas),
        consulta::divisor(),
        consulta::negrito(tr("Também nesta versão"), 18.),
        tambem,
    ].spacing(17);
    let rodape = row![
        Space::new(Length::FILL, 0.0),
        button(row![icones::icone("link", 15.), text(tr("Ver no GitHub")).size(12)].spacing(6).align(Align::Center))
            .padding([8, 13]).style(consulta::estilo_cartao).on_press(acoes.ver_no_github),
        button(text("OK").size(12)).padding([8, 18]).style(tema::estilo_botao_primario).on_press(acoes.fechar),
    ].spacing(10).align(Align::Center);
    consulta::janela(consulta::lateral(nav, None), conteudo.into(), Some(rodape.into()))
}

/// Confere a lista para o teste de cada app: a versão desta compilação no topo, ícones que
/// existem e descrições que cabem no cartão. `Err` diz o que falta.
pub fn conferir(versoes: &[Versao], atual: &str) -> Result<(), String> {
    if versoes.first().map(|v| v.numero) != Some(atual) {
        return Err(format!("acrescente as novidades da {atual} no topo da lista de versões"));
    }
    for v in versoes {
        for (icone, _, descricao) in v.destaques {
            if !icones::existe(icone) { return Err(format!("{}: ícone {icone} não existe", v.numero)); }
            if descricao.chars().count() > 90 { return Err(format!("{}: descrição longa demais: {descricao}", v.numero)); }
        }
    }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn primeira_vez_atualizou_e_nada() {
        let (mut vistos, mut vista) = (false, String::new());
        assert_eq!(ao_abrir(false, &mut vistos, &mut vista, "1.0"), Some(Tela::PrimeirosPassos));
        assert_eq!((vistos, vista.as_str()), (true, "1.0"));
        assert_eq!(ao_abrir(true, &mut vistos, &mut vista, "1.0"), None);
        assert_eq!(ao_abrir(true, &mut vistos, &mut vista, "1.1"), Some(Tela::Novidades));
        // Quem usava antes de existir `primeiros_passos_vistos` está atualizando.
        let (mut vistos, mut vista) = (false, String::new());
        assert_eq!(ao_abrir(true, &mut vistos, &mut vista, "1.1"), Some(Tela::Novidades));
    }

    #[test]
    fn conferir_pede_a_versao_no_topo() {
        const V: &[Versao] = &[Versao { numero: "1.1", data: "", resumo: "", ilustracao: Ilustracao::Atualizacao,
            destaques: &[("check", "a", "b")], tambem: &[] }];
        assert!(conferir(V, "1.1").is_ok());
        assert!(conferir(V, "1.2").is_err());
    }
}
