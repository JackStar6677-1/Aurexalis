# ADR 0004: Floorp Como Submodulo Auditado

## Estado

Aceptado.

## Contexto

Aurexalis necesita estudiar Floorp de forma reproducible para reutilizar su base
Gecko, su build system y su soporte de Chrome Web Store sin copiar parches a
ciegas ni mezclar codigo externo dentro del monorepo.

## Decision

Se integra `Floorp-Projects/Floorp` como submodulo Git en `vendor/floorp`,
inicialmente con clon superficial y revision fijada.

La primera revision registrada es:

```text
f9f9e347588173c5edae2d14e76a13fdbd1284d4
```

## Consecuencias

- El repositorio Aurexalis mantiene una referencia auditable a Floorp.
- Las actualizaciones de Floorp quedan visibles como cambios del submodulo.
- El codigo externo conserva su historial y licencia originales.
- Los parches Aurexalis deben vivir fuera del submodulo hasta decidir un fork
  completo.
- El checkout inicial requiere `git submodule update --init --depth 1`.

## Reglas

- No guardar secretos, certificados ni artefactos de firma de Floorp.
- No editar el submodulo como si fuera codigo propio sin una rama/fork clara.
- Documentar cada area estudiada antes de portar codigo.
- Ejecutar pruebas del workspace Aurexalis despues de cada cambio de integracion.
