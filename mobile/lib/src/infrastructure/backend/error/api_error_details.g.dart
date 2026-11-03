// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'api_error_details.dart';

// **************************************************************************
// CopyWithGenerator
// **************************************************************************

abstract class _$ApiErrorDetailsCWProxy {
  ApiErrorDetails code(String code);

  ApiErrorDetails message(String message);

  ApiErrorDetails details(Object? details);

  /// Creates a new instance with the provided field values.
  /// Omitted fields keep their values; explicit `null` clears nullable fields.
  /// The public API rejects `null` for non-nullable fields. To update a single field use `ApiErrorDetails(...).copyWith.fieldName(value)`.
  ///
  /// Example:
  /// ```dart
  /// ApiErrorDetails(...).copyWith(id: 12, name: "My name")
  /// ```
  ApiErrorDetails call({String code, String message, Object? details});
}

/// Callable proxy for `copyWith` functionality.
/// Use as `instanceOfApiErrorDetails.copyWith(...)` or call `instanceOfApiErrorDetails.copyWith.fieldName(value)` for a single field.
class _$ApiErrorDetailsCWProxyImpl implements _$ApiErrorDetailsCWProxy {
  const _$ApiErrorDetailsCWProxyImpl(this._value);

  final ApiErrorDetails _value;

  @override
  ApiErrorDetails code(String code) => call(code: code);

  @override
  ApiErrorDetails message(String message) => call(message: message);

  @override
  ApiErrorDetails details(Object? details) => call(details: details);

  /// Creates a new instance with the provided field values.
  /// Omitted fields keep their values; explicit `null` clears nullable fields.
  /// The public API rejects `null` for non-nullable fields. To update a single field use `ApiErrorDetails(...).copyWith.fieldName(value)`.
  ///
  /// Example:
  /// ```dart
  /// ApiErrorDetails(...).copyWith(id: 12, name: "My name")
  /// ```
  @override
  ApiErrorDetails call({
    Object? code = const $CopyWithPlaceholder(),
    Object? message = const $CopyWithPlaceholder(),
    Object? details = const $CopyWithPlaceholder(),
  }) {
    return ApiErrorDetails(
      code: code == const $CopyWithPlaceholder() || code == null
          ? _value.code
          // ignore: cast_nullable_to_non_nullable
          : code as String,
      message: message == const $CopyWithPlaceholder() || message == null
          ? _value.message
          // ignore: cast_nullable_to_non_nullable
          : message as String,
      details: details == const $CopyWithPlaceholder()
          ? _value.details
          // ignore: cast_nullable_to_non_nullable
          : details,
    );
  }
}

extension $ApiErrorDetailsCopyWith on ApiErrorDetails {
  /// Returns a callable class used to build a new instance with modified fields.
  /// Example: `instanceOfApiErrorDetails.copyWith(...)` or `instanceOfApiErrorDetails.copyWith.fieldName(...)`.
  // ignore: library_private_types_in_public_api
  _$ApiErrorDetailsCWProxy get copyWith => _$ApiErrorDetailsCWProxyImpl(this);
}

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

ApiErrorDetails _$ApiErrorDetailsFromJson(Map<String, dynamic> json) =>
    $checkedCreate('ApiErrorDetails', json, ($checkedConvert) {
      $checkKeys(json, requiredKeys: const ['code', 'message', 'details']);
      final val = ApiErrorDetails(
        code: $checkedConvert('code', (v) => v as String),
        message: $checkedConvert('message', (v) => v as String),
        details: $checkedConvert('details', (v) => v),
      );
      return val;
    });

Map<String, dynamic> _$ApiErrorDetailsToJson(ApiErrorDetails instance) =>
    <String, dynamic>{
      'code': instance.code,
      'message': instance.message,
      'details': instance.details,
    };
