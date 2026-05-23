# ADR 0001: Base Gecko/Floorp

## Estado

Aceptada.

## Contexto

Aurexalis busca compatibilidad web moderna sin convertirse en otro fork pesado de Chromium. Construir un motor desde cero no es viable para el alcance del proyecto y desviaria el foco hacia compatibilidad basica en vez de producto.

## Decision

La base tecnica sera Gecko, preferentemente mediante Floorp cuando sus parches aporten ventajas reales para UI, personalizacion o compatibilidad con extensiones de Chrome Web Store.

## Consecuencias

- Se hereda compatibilidad web madura.
- El esfuerzo se concentra en shell, UX, bloqueo, importacion y servicios nativos.
- Las integraciones profundas deben respetar las fronteras de Gecko/Floorp.
- Cualquier cambio al core debe pasar por ADR propia antes de implementarse.

