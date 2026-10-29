//! Gestão de janelas auxiliares usada por todos os programas FeatherWorks.
//! O documento e os comandos continuam sendo responsabilidade do aplicativo.
use pelicano::{Settings, Size, Task, WindowCmd, WindowId};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Um número novo por janela aberta; o 0 é a principal (`WindowId::MAIN`).
fn novo_id() -> WindowId {
    static PROXIMO: AtomicU64 = AtomicU64::new(1);
    WindowId(PROXIMO.fetch_add(1, Ordering::Relaxed))
}

/// Uma única janela por função (Configurações, Monitor, Ajuda).
#[derive(Debug)]
pub struct Janelas<T: Copy + Eq> {
    pub principal: Option<WindowId>,
    auxiliares: Vec<(WindowId, T)>,
}
impl<T: Copy + Eq> Default for Janelas<T> {
    fn default() -> Self {
        Self { principal: None, auxiliares: Vec::new() }
    }
}
impl<T: Copy + Eq> Janelas<T> {
    pub fn tipo(&self, id: WindowId) -> Option<T> {
        self.auxiliares.iter().find(|(i, _)| *i == id).map(|(_, tipo)| *tipo)
    }
    pub fn id(&self, tipo: T) -> Option<WindowId> {
        self.auxiliares.iter().find(|(_, t)| *t == tipo).map(|(i, _)| *i)
    }
    pub fn aberta(&self, tipo: T) -> bool {
        self.id(tipo).is_some()
    }
    pub fn registrar(&mut self, id: WindowId, tipo: T) {
        self.auxiliares.retain(|(i, t)| *i != id && *t != tipo);
        self.auxiliares.push((id, tipo));
    }
    pub fn retirar(&mut self, id: WindowId) -> Option<T> {
        let pos = self.auxiliares.iter().position(|(i, _)| *i == id)?;
        Some(self.auxiliares.remove(pos).1)
    }
    pub fn fechar_tipo<M: Send + 'static>(&mut self, tipo: T) -> Task<M> {
        match self.id(tipo) {
            Some(id) => {
                self.retirar(id);
                Task::window_of(id, WindowCmd::Close)
            }
            None => Task::none(),
        }
    }
    /// Replica o comportamento do Slide: clicar no botão alterna a janela.
    /// O id da janela aberta fica em `self.id(tipo)` (o app o vê no `view_window`).
    pub fn alternar<M: Send + 'static>(&mut self, tipo: T, tamanho: Size, barra_propria: bool) -> Task<M> {
        if let Some(id) = self.id(tipo) {
            self.retirar(id);
            return Task::window_of(id, WindowCmd::Close);
        }
        let id = novo_id();
        self.registrar(id, tipo);
        Task::open_window(id, crate::marca::preparar_janela(Settings {
            size: tamanho,
            decorations: !barra_propria,
            close_on_request: false,
            // Os cantos arredondados da moldura (cromo::moldura_redimensionavel_com).
            transparent: barra_propria,
            ..Default::default()
        }))
    }
}

/// Arrastar pelo título; dois cliques rápidos maximizam/restauram.
pub fn arrastar_ou_maximizar<M: Send + 'static>(id: WindowId, ultimo: &mut Option<Instant>) -> Task<M> {
    let duplo = ultimo.is_some_and(|t| t.elapsed().as_millis() < 400);
    *ultimo = if duplo { None } else { Some(Instant::now()) };
    Task::window_of(id, if duplo { WindowCmd::ToggleMaximize } else { WindowCmd::Drag })
}
#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn identifica_e_fecha_auxiliares_sem_confundir_a_principal() {
        let mut j = Janelas::<u8>::default();
        let principal = WindowId::MAIN;
        j.principal = Some(principal);
        let _: Task<()> = j.alternar(1, Size::new(400.0, 300.0), false);
        let config = j.id(1).expect("registrada ao abrir");
        assert_ne!(config, principal);
        assert_eq!(j.tipo(principal), None);
        assert_eq!(j.tipo(config), Some(1));
        assert!(j.aberta(1));
        assert_eq!(j.retirar(config), Some(1));
        assert!(!j.aberta(1));
        assert_eq!(j.principal, Some(principal));
    }
}
