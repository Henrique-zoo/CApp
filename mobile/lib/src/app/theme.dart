/// Configuração visual compartilhada pela aplicação.
///
/// Pertence à camada app porque define a identidade visual usada pelos
/// módulos. As telas consomem o tema sem precisar repetir sua paleta.
///
/// Esta biblioteca apenas constrói configurações; não executa I/O.
library;

import 'package:flutter/material.dart';

/// Constrói o tema escuro do CApp.
///
/// Retorna uma configuração Material 3 com cores derivadas da cor principal.
/// Os componentes Material utilizam esse esquema para textos, superfícies
/// e estados de interação.
ThemeData createDarkTheme() {
  return ThemeData(
    useMaterial3: true,
    colorScheme: ColorScheme.fromSeed(
      seedColor: const Color(0xFF8B5CF6),
      brightness: Brightness.dark,
    ),
    scaffoldBackgroundColor: const Color(0xFF101116),
  );
}
