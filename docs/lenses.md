# Lenses

A `show` line picks a lens. A lens turns the facts of a file into a picture.
It binds a field of the facts to each channel of the picture, and those
bindings are its dials. The file states facts and never a position.

This page lists each lens that this version draws. A test makes the table from
`horsegirl_core::lens`, so the table and the program agree.

<!-- lenses: start -->
| Lens | Question | Dials | Needs | Solver |
| --- | --- | --- | --- | --- |
| `flow` | What depends on what. | `x: depth`, `inside: group`, `colour: owner` | links | layered |
<!-- lenses: end -->

## Channels and fields

A channel is a property of the picture. A field is what the facts say.

| Channel | What it sets |
| --- | --- |
| `x` | The position from left to right. |
| `inside` | The box that holds a thing. |
| `colour` | The palette slot of a thing. |

| Field | What it reads |
| --- | --- |
| `depth` | The longest path of links that leads to a thing. |
| `group` | The thing that holds a thing, as braces state it. |
| `owner` | The value of the `owner` fact. |

## flow

The flow lens answers the question "what depends on what". A thing stands to
the right of each thing that links to it, so the picture reads from left to
right. A group is a box around the things it holds. Each thing takes the
palette slot of its `owner`: the first three owners take slots 1 to 3, and a
fourth owner takes slot 1 again.

```horsegirl
Web app, Mobile app -> API gateway
API gateway -> Orders
show flow
```

The flow lens orders things by their links. A file with `show flow` and no link
gives the error `E0301`, because there is nothing to order. A `-x` rule is not
a link.

### A link to a group

A link can start or end at a group:

```horsegirl
API gateway -> Services
Services -> Postgres
Services {
  Orders, Payments
}
show flow
```

The flow lens draws `API gateway -> Services` to the left border of the box of
`Services`, and `Services -> Postgres` from its right border. Such a link
enters no box inside the group. The lens ranks the group as one unit for the
link: each thing in `Services` stands to the right of `API gateway`, and to the
left of `Postgres`.

Other links can put things of a group on both sides of the other end of the
link. With `Orders -> Audit` and `Audit -> Payments`, `Orders` stands left of
`Audit` and `Payments` right of it. `Services` then has no border that faces
`Audit`, so `Audit -> Services` ends at a point inside the group box, outside
every box that the group holds.

A link between a group and a thing inside it, at any depth, or between a group
and itself, is not drawn. The file states that a box links to its own
contents, and the picture has no line for it. The check gives the warning
`W0110` for each such link, so the writer can link a thing outside the group
or two things inside it.
