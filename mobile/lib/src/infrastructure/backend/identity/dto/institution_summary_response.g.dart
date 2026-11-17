// GENERATED CODE - DO NOT MODIFY BY HAND

// ignore_for_file: unnecessary_cast

part of 'institution_summary_response.dart';

// **************************************************************************
// CopyWithGenerator
// **************************************************************************

abstract class _$InstitutionSummaryResponseCWProxy {
  InstitutionSummaryResponse id(UuidValue id);

  InstitutionSummaryResponse name(String name);

  InstitutionSummaryResponse acronym(String? acronym);

  /// Creates a new instance with the provided field values.
  /// Omitted fields keep their values; explicit `null` clears nullable fields.
  /// The public API rejects `null` for non-nullable fields. To update a single field use `InstitutionSummaryResponse(...).copyWith.fieldName(value)`.
  ///
  /// Example:
  /// ```dart
  /// InstitutionSummaryResponse(...).copyWith(id: 12, name: "My name")
  /// ```
  InstitutionSummaryResponse call({UuidValue id, String name, String? acronym});
}

/// Callable proxy for `copyWith` functionality.
/// Use as `instanceOfInstitutionSummaryResponse.copyWith(...)` or call `instanceOfInstitutionSummaryResponse.copyWith.fieldName(value)` for a single field.
class _$InstitutionSummaryResponseCWProxyImpl
    implements _$InstitutionSummaryResponseCWProxy {
  const _$InstitutionSummaryResponseCWProxyImpl(this._value);

  final InstitutionSummaryResponse _value;

  @override
  InstitutionSummaryResponse id(UuidValue id) => call(id: id);

  @override
  InstitutionSummaryResponse name(String name) => call(name: name);

  @override
  InstitutionSummaryResponse acronym(String? acronym) => call(acronym: acronym);

  /// Creates a new instance with the provided field values.
  /// Omitted fields keep their values; explicit `null` clears nullable fields.
  /// The public API rejects `null` for non-nullable fields. To update a single field use `InstitutionSummaryResponse(...).copyWith.fieldName(value)`.
  ///
  /// Example:
  /// ```dart
  /// InstitutionSummaryResponse(...).copyWith(id: 12, name: "My name")
  /// ```
  @override
  InstitutionSummaryResponse call({
    Object? id = const $CopyWithPlaceholder(),
    Object? name = const $CopyWithPlaceholder(),
    Object? acronym = const $CopyWithPlaceholder(),
  }) {
    return InstitutionSummaryResponse(
      id: id == const $CopyWithPlaceholder() || id == null
          ? _value.id
          // ignore: cast_nullable_to_non_nullable
          : id as UuidValue,
      name: name == const $CopyWithPlaceholder() || name == null
          ? _value.name
          // ignore: cast_nullable_to_non_nullable
          : name as String,
      acronym: acronym == const $CopyWithPlaceholder()
          ? _value.acronym
          // ignore: cast_nullable_to_non_nullable
          : acronym as String?,
    );
  }
}

extension $InstitutionSummaryResponseCopyWith on InstitutionSummaryResponse {
  /// Returns a callable class used to build a new instance with modified fields.
  /// Example: `instanceOfInstitutionSummaryResponse.copyWith(...)` or `instanceOfInstitutionSummaryResponse.copyWith.fieldName(...)`.
  // ignore: library_private_types_in_public_api
  _$InstitutionSummaryResponseCWProxy get copyWith =>
      _$InstitutionSummaryResponseCWProxyImpl(this);
}

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

InstitutionSummaryResponse _$InstitutionSummaryResponseFromJson(
  Map<String, dynamic> json,
) => $checkedCreate('InstitutionSummaryResponse', json, ($checkedConvert) {
  $checkKeys(json, requiredKeys: const ['id', 'name', 'acronym']);
  final val = InstitutionSummaryResponse(
    id: $checkedConvert(
      'id',
      (v) => const UuidValueConverter().fromJson(v as String),
    ),
    name: $checkedConvert('name', (v) => v as String),
    acronym: $checkedConvert('acronym', (v) => v as String?),
  );
  return val;
});

Map<String, dynamic> _$InstitutionSummaryResponseToJson(
  InstitutionSummaryResponse instance,
) => <String, dynamic>{
  'id': const UuidValueConverter().toJson(instance.id),
  'name': instance.name,
  'acronym': instance.acronym,
};
