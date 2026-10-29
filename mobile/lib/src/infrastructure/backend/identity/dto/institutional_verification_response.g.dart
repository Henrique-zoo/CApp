// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'institutional_verification_response.dart';

// **************************************************************************
// CopyWithGenerator
// **************************************************************************

abstract class _$InstitutionalVerificationResponseCWProxy {
  InstitutionalVerificationResponse profile(
    InstitutionalIdentityResponse profile,
  );

  InstitutionalVerificationResponse expiresAt(DateTime expiresAt);

  /// Creates a new instance with the provided field values.
  /// Omitted fields keep their values; explicit `null` clears nullable fields.
  /// The public API rejects `null` for non-nullable fields. To update a single field use `InstitutionalVerificationResponse(...).copyWith.fieldName(value)`.
  ///
  /// Example:
  /// ```dart
  /// InstitutionalVerificationResponse(...).copyWith(id: 12, name: "My name")
  /// ```
  InstitutionalVerificationResponse call({
    InstitutionalIdentityResponse profile,
    DateTime expiresAt,
  });
}

/// Callable proxy for `copyWith` functionality.
/// Use as `instanceOfInstitutionalVerificationResponse.copyWith(...)` or call `instanceOfInstitutionalVerificationResponse.copyWith.fieldName(value)` for a single field.
class _$InstitutionalVerificationResponseCWProxyImpl
    implements _$InstitutionalVerificationResponseCWProxy {
  const _$InstitutionalVerificationResponseCWProxyImpl(this._value);

  final InstitutionalVerificationResponse _value;

  @override
  InstitutionalVerificationResponse profile(
    InstitutionalIdentityResponse profile,
  ) => call(profile: profile);

  @override
  InstitutionalVerificationResponse expiresAt(DateTime expiresAt) =>
      call(expiresAt: expiresAt);

  /// Creates a new instance with the provided field values.
  /// Omitted fields keep their values; explicit `null` clears nullable fields.
  /// The public API rejects `null` for non-nullable fields. To update a single field use `InstitutionalVerificationResponse(...).copyWith.fieldName(value)`.
  ///
  /// Example:
  /// ```dart
  /// InstitutionalVerificationResponse(...).copyWith(id: 12, name: "My name")
  /// ```
  @override
  InstitutionalVerificationResponse call({
    Object? profile = const $CopyWithPlaceholder(),
    Object? expiresAt = const $CopyWithPlaceholder(),
  }) {
    return InstitutionalVerificationResponse(
      profile: profile == const $CopyWithPlaceholder() || profile == null
          ? _value.profile
          // ignore: cast_nullable_to_non_nullable
          : profile as InstitutionalIdentityResponse,
      expiresAt: expiresAt == const $CopyWithPlaceholder() || expiresAt == null
          ? _value.expiresAt
          // ignore: cast_nullable_to_non_nullable
          : expiresAt as DateTime,
    );
  }
}

extension $InstitutionalVerificationResponseCopyWith
    on InstitutionalVerificationResponse {
  /// Returns a callable class used to build a new instance with modified fields.
  /// Example: `instanceOfInstitutionalVerificationResponse.copyWith(...)` or `instanceOfInstitutionalVerificationResponse.copyWith.fieldName(...)`.
  // ignore: library_private_types_in_public_api
  _$InstitutionalVerificationResponseCWProxy get copyWith =>
      _$InstitutionalVerificationResponseCWProxyImpl(this);
}

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

InstitutionalVerificationResponse _$InstitutionalVerificationResponseFromJson(
  Map<String, dynamic> json,
) => $checkedCreate('InstitutionalVerificationResponse', json, (
  $checkedConvert,
) {
  $checkKeys(json, requiredKeys: const ['profile', 'expiresAt']);
  final val = InstitutionalVerificationResponse(
    profile: $checkedConvert(
      'profile',
      (v) => InstitutionalIdentityResponse.fromJson(v as Map<String, dynamic>),
    ),
    expiresAt: $checkedConvert('expiresAt', (v) => DateTime.parse(v as String)),
  );
  return val;
});

Map<String, dynamic> _$InstitutionalVerificationResponseToJson(
  InstitutionalVerificationResponse instance,
) => <String, dynamic>{
  'profile': instance.profile.toJson(),
  'expiresAt': instance.expiresAt.toIso8601String(),
};
