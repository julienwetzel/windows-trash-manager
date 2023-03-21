#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::TemplateApp;

pub const NOTICE: &str =
    "    /\\_/\\  (\n   ( ^.^ ) _)\n      \\\"/  (\n   (  |  |  ) \n(__d b__)\n\n\
📖 NOTICE D'UTILISATION 📖\n\n\
▶ Préserver X jours ◀\n\
  Permet de spécifier le nombre de jours durant lesquels les éléments restent \
  dans la corbeille avant d'être supprimés définitivement.\n\
📺 Exemple: Si défini sur 5 jours, les éléments supprimés il y a plus de 5 jours \
  seront automatiquement supprimés de la corbeille.\n\
▶ Analyser ◀\n\
  Ce bouton permet de lister les éléments supprimés au-delà du nombre de jours \
  défini pour la corbeille.\n\
▶ Supprimer définitivement ◀\n\
  Ce bouton permet de supprimer définitivement les éléments qui ont dépassé le \
  nombre de jours défini pour la corbeille.\n\
🖊 Remarque: Il n'est pas nécessaire d'effectuer une analyse au préalable.\n\n";
