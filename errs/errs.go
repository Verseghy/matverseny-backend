package errs

import "errors"

var (
	ErrNotImplemented         = errors.New("E0000: not implemented")
	ErrEmailRequired          = errors.New("E0001: email is required")
	ErrPasswordRequired       = errors.New("E0002: password is required")
	ErrInvalidEmailOrPassword = errors.New("E0003: invalid email or password")
	ErrDatabase               = errors.New("E0004: database error")
	ErrCryptographic          = errors.New("E0005: cryptographic failure")
	ErrJWT                    = errors.New("E0006: JWT failure")
	ErrNameRequired           = errors.New("E0007: name is required")
	ErrEmailAddressFormat     = errors.New("E0008: email address format incorrect")
	ErrSchoolRequired         = errors.New("E0009: school is required")
	ErrAlreadyExists          = errors.New("E0010: user already registered")
	ErrTokenExpired           = errors.New("E0011: token expired")
	ErrUnauthorized           = errors.New("E0012: unauthorized")
	ErrInvalidPosition        = errors.New("E0013: invalid position")
	ErrNotFound               = errors.New("E0014: not found")
	ErrInvalidID              = errors.New("E0015: invalid ID")
	ErrNotAdmin               = errors.New("E0016: not admin")
	ErrNoTeam                 = errors.New("E0017: no team")
	ErrMail                   = errors.New("E0018: error sending email")
	ErrInvalidResetToken      = errors.New("E0019: reset token invalid")
	ErrQueue                  = errors.New("E0020: queue error")
)
