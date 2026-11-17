/// Conversão de UUIDs no contrato JSON do backend.
///
/// Os DTOs representam identificadores como UuidValue; a rede transporta strings.
/// Este conversor verifica o formato hexadecimal e os separadores, sem exigir
/// versão ou variante específica. A interpretação normaliza letras para minúsculas.
/// O formato persistente local possui seu próprio codec e não depende deste tipo.
library;

import 'package:json_annotation/json_annotation.dart';
import 'package:uuid/uuid.dart';

/// Converte UUIDs textuais do JSON, validando seu formato.
class UuidValueConverter implements JsonConverter<UuidValue, String> {
  /// Cria o conversor sem estado para uso nas anotações dos DTOs.
  const UuidValueConverter();

  /// Interpreta [json]; lança [FormatException] para formato incompatível.
  @override
  UuidValue fromJson(String json) => UuidValue.withFormatValidation(json);

  /// Retorna a representação textual do valor, sem nova validação.
  @override
  String toJson(UuidValue value) => value.uuid;
}
