# Improved API Error Handling

This document describes the standardized error handling system implemented to provide better error responses to clients while maintaining security best practices.

## Overview

The new error handling system provides:

1. **Consistent Error Response Format** - All API endpoints return errors in the same JSON format
2. **Appropriate HTTP Status Codes** - Errors are mapped to semantically correct status codes  
3. **Security-Conscious Messages** - Error messages don't expose internal implementation details
4. **Comprehensive Error Logging** - Internal errors are logged and sent to Sentry for debugging
5. **Client-Friendly Error Categories** - Errors are categorized to help clients handle them appropriately

## Error Response Format

All API errors now return responses in this standardized format:

```json
{
  "error": "ERROR_TYPE",
  "message": "Human-readable error message",
  "status_code": 400,
  "details": {
    "field": "Additional context (optional)"
  }
}
```

### Error Fields

- **error**: Machine-readable error type (e.g., `INVALID_INPUT`, `NOT_FOUND`)
- **message**: Human-readable description safe for display to users
- **status_code**: HTTP status code as integer
- **details**: Optional object with additional context (validation errors, etc.)

## Error Categories

### 400 - Invalid Input (`INVALID_INPUT`)
- Client provided malformed or invalid data
- Examples: Invalid JSON, missing required fields, invalid format

### 401 - Unauthorized (`UNAUTHORIZED`) 
- Authentication required or credentials invalid
- Examples: Missing token, expired token, invalid username/password

### 403 - Forbidden (`FORBIDDEN`)
- User authenticated but lacks permission
- Examples: Insufficient role, resource access denied

### 404 - Not Found (`NOT_FOUND`)
- Requested resource doesn't exist
- Examples: Device not found, user not found

### 409 - Conflict (`CONFLICT`)
- Request conflicts with current state
- Examples: Username already taken, duplicate resource

### 422 - Unprocessable Entity (`UNPROCESSABLE_ENTITY`)
- Request semantically incorrect but syntactically valid
- Examples: Business rule violations

### 429 - Too Many Requests (`TOO_MANY_REQUESTS`)
- Rate limit exceeded
- Examples: API rate limiting

### 500 - Internal Error (`INTERNAL_ERROR`)
- Unexpected server error
- Examples: Database connection failed, unexpected exceptions

### 502 - Service Unavailable (`SERVICE_UNAVAILABLE`)
- External service dependency failed
- Examples: Device service down, database unreachable

### 503 - Temporarily Unavailable (`TEMPORARILY_UNAVAILABLE`)
- Service temporarily overloaded or in maintenance
- Examples: High load, scheduled maintenance

## Security Considerations

### What We Hide
- Database schema details
- Internal service architecture  
- Stack traces and debug information
- Specific library/framework versions
- File paths and system information

### What We Expose
- Resource existence (404 vs 403 carefully chosen)
- Validation errors (but not implementation details)
- Service availability status
- Rate limiting information

### Error Logging
- Internal errors are logged with full details for debugging
- Sensitive information is not logged in user-facing messages
- All errors are sent to Sentry for monitoring and alerting

## Implementation Examples

### Service Layer Error Mapping

```rust
impl IntoApiError for AuthError {
    fn into_api_error(self) -> ApiErrorResponse {
        match self {
            AuthError::InvalidCredentials => {
                errors::unauthorized("Invalid username or password")
            }
            AuthError::DatabaseConnection => {
                tracing::error!("Database error: {:?}", self);
                sentry::capture_error(&self);
                errors::internal_error()
            }
            // ... other mappings
        }
    }
}
```

### API Layer Usage

```rust
// Errors are automatically converted to standardized responses
pub async fn get_device(id: Uuid) -> Result<Json<Device>, Error> {
    let device = device_service::get_device(id).await?;
    Ok(Json(device))
}
```

### Client-Side Error Handling

```typescript
// Clients can rely on consistent error format
interface ApiError {
  error: string;
  message: string;
  status_code: number;
  details?: Record<string, string>;
}

// Handle errors by category
function handleApiError(error: ApiError) {
  switch (error.error) {
    case 'UNAUTHORIZED':
      redirectToLogin();
      break;
    case 'NOT_FOUND':
      showNotFoundMessage();
      break;
    case 'INVALID_INPUT':
      highlightValidationErrors(error.details);
      break;
    default:
      showGenericError(error.message);
  }
}
```

## Migration Benefits

### Before
- Most errors returned 500 Internal Server Error
- Error messages exposed internal details
- Inconsistent error formats across endpoints
- Difficult for clients to handle errors appropriately
- Poor debugging experience

### After
- Appropriate HTTP status codes for each error type
- Security-conscious error messages
- Consistent JSON error format
- Clear error categories for client handling
- Comprehensive logging for debugging
- Better user experience with actionable error messages

## Testing Error Responses

Use the following examples to test the new error handling:

```bash
# Test invalid input
curl -X POST /api/device -d '{"invalid": "data"}' \
  -H "Content-Type: application/json"

# Expected: 400 with INVALID_INPUT error

# Test not found
curl /api/device/00000000-0000-0000-0000-000000000000

# Expected: 404 with NOT_FOUND error

# Test unauthorized
curl /api/device/protected-endpoint

# Expected: 401 with UNAUTHORIZED error
```

This new error handling system significantly improves the API's usability for clients while maintaining security and providing better debugging capabilities for developers.