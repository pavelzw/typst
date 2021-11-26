use super::prelude::*;
use super::AlignNode;

/// `place`: Place content at an absolute position.
pub fn place(_: &mut EvalContext, args: &mut Args) -> TypResult<Value> {
    let aligns = args.find().unwrap_or(Spec::new(Some(Align::Left), None));
    let tx = args.named("dx")?.unwrap_or_default();
    let ty = args.named("dy")?.unwrap_or_default();
    let body: Template = args.expect("body")?;
    Ok(Value::Template(Template::from_block(move |style| {
        PlacedNode {
            child: body.pack(style).moved(Point::new(tx, ty)).aligned(aligns),
        }
    })))
}

/// A node that places its child absolutely.
#[derive(Debug, Hash)]
pub struct PlacedNode {
    /// The node to be placed.
    pub child: PackedNode,
}

impl PlacedNode {
    /// Whether this node wants to be placed relative to its its parent's base
    /// origin. instead of relative to the parent's current flow/cursor
    /// position.
    pub fn out_of_flow(&self) -> bool {
        self.child
            .downcast::<AlignNode>()
            .map_or(false, |node| node.aligns.y.is_some())
    }
}

impl Layout for PlacedNode {
    fn layout(
        &self,
        ctx: &mut LayoutContext,
        regions: &Regions,
    ) -> Vec<Constrained<Rc<Frame>>> {
        let out_of_flow = self.out_of_flow();

        // The pod is the base area of the region because for absolute
        // placement we don't really care about the already used area (current).
        let pod = {
            let expand = if out_of_flow { Spec::splat(true) } else { regions.expand };
            Regions::one(regions.base, regions.base, expand)
        };

        let mut frames = self.child.layout(ctx, &pod);
        let Constrained { item: frame, cts } = &mut frames[0];

        // If expansion is off, zero all sizes so that we don't take up any
        // space in our parent. Otherwise, respect the expand settings.
        let target = regions.expand.select(regions.current, Size::zero());
        Rc::make_mut(frame).resize(target, Align::LEFT_TOP);

        // Place relative to parent's base origin by offsetting our elements by
        // the negative cursor position.
        if out_of_flow {
            let offset = (regions.current - regions.base).to_point();
            Rc::make_mut(frame).translate(offset);
        }

        // Set base constraint because our pod size is base and exact
        // constraints if we needed to expand or offset.
        *cts = Constraints::new(regions.expand);
        cts.base = regions.base.map(Some);
        cts.exact = regions.current.filter(regions.expand | out_of_flow);

        frames
    }
}