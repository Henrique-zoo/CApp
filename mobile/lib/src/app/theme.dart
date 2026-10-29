/// Configuração visual compartilhada pela aplicação.
///
/// Pertence à camada app porque define a identidade visual usada pelos
/// módulos. As telas consomem o tema sem precisar repetir sua paleta.
///
/// Esta biblioteca apenas constrói configurações; não executa I/O.
library;

import 'package:flutter/material.dart';

/// Constrói o tema escuro compartilhado pelo CApp.
///
/// Retorna uma configuração Material 3 com a família Inter e cores derivadas
/// da cor principal. Os componentes que herdam o tema utilizam essa tipografia,
/// salvo quando possuem uma família explicitamente definida.
///
/// Os arquivos da fonte são empacotados conforme a declaração no pubspec.
/// Esta função apenas constrói o tema; não realiza downloads ou outras
/// operações de I/O.
ThemeData createDarkTheme() {
  return ThemeData(
    useMaterial3: true,
    fontFamily: 'Inter',
    colorScheme: ColorScheme.fromSeed(
      seedColor: const Color(0xFF8B5CF6),
      brightness: Brightness.dark,
    ),
    scaffoldBackgroundColor: const Color(0xFF101116),
  );
}
