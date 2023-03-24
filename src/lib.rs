#![warn(clippy::all, rust_2021_compatibility)]
mod app;
pub use app::TemplateApp;
mod consts;

pub const NOTICE: &str = "
📖 NOTICE D'UTILISATION 📖\n\n\
▶ Préserver X jours ◀\n\
  Permet de spécifier le nombre de jours durant lesquels les éléments restent \
  dans la corbeille avant d'être supprimés définitivement.\n\
   Exemple: Si défini sur 5 jours, les éléments supprimés il y a plus de 5 jours \
  seront automatiquement supprimés de la corbeille.\n\
▶ Analyser ◀\n\
  Ce bouton permet de lister les éléments supprimés au-delà du nombre de jours \
  défini pour la corbeille.\n\
▶ Supprimer définitivement ◀\n\
  Ce bouton permet de supprimer définitivement les éléments qui ont dépassé le \
  nombre de jours défini pour la corbeille.\n\
🖊 Remarque: Il n'est pas nécessaire d'effectuer une analyse au préalable.\n\n";

pub struct GitHubInfo {
    url: String,
    url_blob: String,
}

impl Default for GitHubInfo {
    fn default() -> Self {
        Self {
            url: consts::GITHUB_URL.to_owned(),
            url_blob: consts::GITHUB_URL_BLOB.to_owned(),
        }
    }
}

pub struct CircularBuffer<T> {
    buffer: Vec<T>,
    head: usize,
    tail: usize,
}

impl<T> CircularBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        assert!(
            capacity > 0,
            "[CircularBuffer] La capacité doit être supérieure à zéro."
        );
        Self {
            buffer: Vec::with_capacity(capacity),
            head: 0,
            tail: 0,
        }
    }

    pub fn push(&mut self, item: T) {
        assert!(
            self.buffer.capacity() > 0,
            "[CircularBuffer] Le tampon doit avoir une capacité supérieure à zéro."
        );
        if self.buffer.len() < self.buffer.capacity() {
            self.buffer.push(item);
            self.tail += 1;
        } else {
            self.buffer[self.head] = item;
            self.head = (self.head + 1) % self.buffer.capacity();
            self.tail = (self.tail + 1) % self.buffer.capacity();
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        let len = self.buffer.len();
        let cap = self.buffer.capacity();
        let tail = self.tail;

        self.buffer
            .iter()
            .skip(self.head)
            .chain(self.buffer.iter().take(tail))
            .cycle()
            .take(len.min(cap))
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.head = 0;
        self.tail = 0;
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}
