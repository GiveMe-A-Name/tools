use crate::{registry::RuleRoot, FromServices, Queryable, Rule, RuleKey, ServiceBag};
use rome_diagnostics::{Error, Result};
use std::ops::Deref;

type RuleQueryResult<R> = <<R as Rule>::Query as Queryable>::Output;
type RuleServiceBag<R> = <<R as Rule>::Query as Queryable>::Services;

#[derive(Clone)]
pub struct ServiceBagRuleOptionsWrapper<R: Rule>(pub R::Options);

pub struct RuleContext<'a, R>
where
    R: ?Sized + Rule,
{
    query_result: &'a RuleQueryResult<R>,
    root: &'a RuleRoot<R>,
    bag: &'a ServiceBag,
    services: RuleServiceBag<R>,
    globals: &'a [&'a str],
}

impl<'a, R> RuleContext<'a, R>
where
    R: Rule + Sized + 'static,
{
    pub fn new(
        query_result: &'a RuleQueryResult<R>,
        root: &'a RuleRoot<R>,
        services: &'a ServiceBag,
        globals: &'a [&'a str],
    ) -> Result<Self, Error> {
        let rule_key = RuleKey::rule::<R>();
        Ok(Self {
            query_result,
            root,
            bag: services,
            services: FromServices::from_services(&rule_key, services)?,
            globals,
        })
    }

    pub fn query(&self) -> &RuleQueryResult<R> {
        self.query_result
    }

    /// Returns a clone of the AST root
    pub fn root(&self) -> RuleRoot<R> {
        self.root.clone()
    }

    /// It retrieves the options that belong to a rule, if they exist.
    ///
    /// In order to retrieve a typed data structure, you have to create a deserializable
    /// data structure and define it inside the generic type `type Options` of the [Rule]
    ///
    /// ## Examples
    ///
    /// ```rust,ignore
    /// use rome_analyze::{declare_rule, Rule, RuleCategory, RuleMeta, RuleMetadata};
    /// use rome_analyze::context::RuleContext;
    /// use serde::Deserialize;
    /// declare_rule! {
    ///     /// Some doc
    ///     pub(crate) Name {
    ///         version: "0.0.0",
    ///         name: "name",
    ///         recommended: true,
    ///     }
    /// }
    ///
    /// #[derive(Deserialize)]
    /// struct RuleOptions {}
    ///
    /// impl Rule for Name {
    ///     const CATEGORY: RuleCategory = RuleCategory::Lint;
    ///     type Query = ();
    ///     type State = ();
    ///     type Signals = ();
    ///     type Options = RuleOptions;
    ///
    ///     fn run(ctx: &RuleContext<Self>) -> Self::Signals {
    ///         if let Some(options) = ctx.options() {
    ///             // do something with the options now
    ///         }
    ///     }
    /// }
    /// ```
    pub fn options(&self) -> &R::Options {
        let ServiceBagRuleOptionsWrapper(options) = self
            .bag
            .get_service::<ServiceBagRuleOptionsWrapper<R>>()
            .unwrap();
        options
    }

    /// Checks whether the provided text belongs to globals
    pub fn is_global(&self, text: &str) -> bool {
        self.globals.contains(&text)
    }
}

impl<'a, R> Deref for RuleContext<'a, R>
where
    R: Rule,
{
    type Target = RuleServiceBag<R>;

    fn deref(&self) -> &Self::Target {
        &self.services
    }
}
