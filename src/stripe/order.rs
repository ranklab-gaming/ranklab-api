use serde::{Deserialize, Serialize};

use crate::stripe::ids::OrderId;
use stripe::{Address, TaxRate};
use stripe::{Application, Expandable, Metadata, Object, Timestamp};
use stripe::{BillingDetails, Currency, Customer, Discount, TaxIdType};
use stripe::{Client, Response};

use super::order_line_item::OrderLineItem;

/// The `Expand` struct is used to serialize `expand` arguments in retrieve and list apis.
#[doc(hidden)]
#[derive(Serialize)]
pub struct Expand<'a> {
  #[serde(skip_serializing_if = "Expand::is_empty")]
  pub expand: &'a [&'a str],
}

impl Expand<'_> {
  pub(crate) fn is_empty(expand: &[&str]) -> bool {
    expand.is_empty()
  }
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
  Open,
  Processing,
  Complete,
  Canceled,
}

impl OrderStatus {
  pub fn as_str(self) -> &'static str {
    match self {
      OrderStatus::Open => "open",
      OrderStatus::Processing => "processing",
      OrderStatus::Complete => "complete",
      OrderStatus::Canceled => "canceled",
    }
  }
}

impl AsRef<str> for OrderStatus {
  fn as_ref(&self) -> &str {
    self.as_str()
  }
}

impl std::fmt::Display for OrderStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    self.as_str().fmt(f)
  }
}
impl std::default::Default for OrderStatus {
  fn default() -> Self {
    Self::Open
  }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct OrderAutomaticTax {
  /// Indicates whether automatic tax is enabled for the session.
  pub enabled: bool,

  /// The status of the most recent automated tax calculation for this session.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<OrderAutomaticTaxStatus>,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OrderAutomaticTaxStatus {
  Complete,
  Failed,
  RequiresLocationInputs,
}

impl OrderAutomaticTaxStatus {
  pub fn as_str(self) -> &'static str {
    match self {
      OrderAutomaticTaxStatus::Complete => "complete",
      OrderAutomaticTaxStatus::Failed => "failed",
      OrderAutomaticTaxStatus::RequiresLocationInputs => "requires_location_inputs",
    }
  }
}

impl AsRef<str> for OrderAutomaticTaxStatus {
  fn as_ref(&self) -> &str {
    self.as_str()
  }
}

impl std::fmt::Display for OrderAutomaticTaxStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    self.as_str().fmt(f)
  }
}

impl std::default::Default for OrderAutomaticTaxStatus {
  fn default() -> Self {
    Self::Complete
  }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct OrderShippingDetails {
  /// Billing address.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub address: Option<Address>,

  /// Full name.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,

  /// Billing phone number (including extension).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub phone: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct OrderTotalDetailsBreakdown {
  /// The aggregated line item discounts.
  pub discounts: Vec<OrderTotalDetailsBreakdownDiscount>,

  /// The aggregated line item tax amounts by rate.
  pub taxes: Vec<OrderTotalDetailsTax>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LineItemsDiscountAmount {
  /// The amount discounted.
  pub amount: i64,

  pub discount: Discount,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LineItemsTaxAmount {
  /// Amount of tax applied for this rate.
  pub amount: i64,

  pub rate: TaxRate,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct OrderTotalDetails {
  /// This is the sum of all the line item discounts.
  pub amount_discount: i64,

  /// This is the sum of all the line item shipping amounts.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub amount_shipping: i64,

  /// This is the sum of all the line item tax amounts.
  pub amount_tax: i64,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub breakdown: Option<OrderTotalDetailsBreakdown>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct OrderTaxId {
  /// The type of the tax ID, one of `eu_vat`, `br_cnpj`, `br_cpf`, `gb_vat`, `nz_gst`, `au_abn`, `au_arn`, `in_gst`, `no_vat`, `za_vat`, `ch_vat`, `mx_rfc`, `sg_uen`, `ru_inn`, `ru_kpp`, `ca_bn`, `hk_br`, `es_cif`, `tw_vat`, `th_vat`, `jp_cn`, `jp_rn`, `li_uid`, `my_itn`, `us_ein`, `kr_brn`, `ca_qst`, `ca_gst_hst`, `ca_pst_bc`, `ca_pst_mb`, `ca_pst_sk`, `my_sst`, `sg_gst`, `ae_trn`, `cl_tin`, `sa_vat`, `id_npwp`, `my_frp`, `il_vat`, `ge_vat`, `ua_vat`, `is_vat`, or `unknown`.
  #[serde(rename = "type")]
  pub type_: TaxIdType,

  /// The value of the tax ID.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub value: Option<String>,
}

/// An enum representing the possible values of an `Order`'s `tax_exempt` field.
#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OrderTaxExempt {
  Exempt,
  None,
  Reverse,
}

impl OrderTaxExempt {
  pub fn as_str(self) -> &'static str {
    match self {
      OrderTaxExempt::Exempt => "exempt",
      OrderTaxExempt::None => "none",
      OrderTaxExempt::Reverse => "reverse",
    }
  }
}

impl AsRef<str> for OrderTaxExempt {
  fn as_ref(&self) -> &str {
    self.as_str()
  }
}

impl std::fmt::Display for OrderTaxExempt {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    self.as_str().fmt(f)
  }
}
impl std::default::Default for OrderTaxExempt {
  fn default() -> Self {
    Self::Exempt
  }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct OrderTaxDetails {
  /// The type of the tax ID, one of `eu_vat`, `br_cnpj`, `br_cpf`, `gb_vat`, `nz_gst`, `au_abn`, `au_arn`, `in_gst`, `no_vat`, `za_vat`, `ch_vat`, `mx_rfc`, `sg_uen`, `ru_inn`, `ru_kpp`, `ca_bn`, `hk_br`, `es_cif`, `tw_vat`, `th_vat`, `jp_cn`, `jp_rn`, `li_uid`, `my_itn`, `us_ein`, `kr_brn`, `ca_qst`, `ca_gst_hst`, `ca_pst_bc`, `ca_pst_mb`, `ca_pst_sk`, `my_sst`, `sg_gst`, `ae_trn`, `cl_tin`, `sa_vat`, `id_npwp`, `my_frp`, `il_vat`, `ge_vat`, `ua_vat`, `is_vat`, or `unknown`.
  #[serde(rename = "type")]
  pub tax_exempt: Option<OrderTaxExempt>,

  /// The value of the tax ID.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub value: Option<String>,
}

/// The resource representing a Stripe "Order".
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Order {
  /// Unique identifier for the object.
  pub id: OrderId,

  pub amount_total: i64,

  pub amount_subtotal: i64,

  /// ID of the Connect Application that created the order.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub application: Option<Expandable<Application>>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub automatic_tax: Option<OrderAutomaticTax>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub billing_details: Option<BillingDetails>,

  /// Time at which the object was created.
  ///
  /// Measured in seconds since the Unix epoch.
  pub created: Timestamp,

  /// Three-letter [ISO currency code](https://www.iso.org/iso-4217-currency-codes.html), in lowercase.
  ///
  /// Must be a [supported currency](https://stripe.com/docs/currencies).
  pub currency: Currency,

  /// The customer used for the order.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub customer: Option<Expandable<Customer>>,

  /// An arbitrary string attached to the object.
  ///
  /// Often useful for displaying to users.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub discounts: Option<Vec<Expandable<Discount>>>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub ip_address: Option<String>,

  /// Has the value `true` if the object exists in live mode or the value `false` if the object exists in test mode.
  pub livemode: bool,

  /// List of items constituting the order.
  ///
  /// An order can have up to 25 items.
  pub line_items: Vec<Expandable<OrderLineItem>>,

  /// Set of [key-value pairs](https://stripe.com/docs/api/metadata) that you can attach to an object.
  ///
  /// This can be useful for storing additional information about the object in a structured format.
  #[serde(default)]
  pub metadata: Metadata,

  /// Current order status.
  ///
  /// One of `created`, `paid`, `canceled`, `fulfilled`, or `returned`.
  /// More details in the [Orders Guide](https://stripe.com/docs/orders/guide#understanding-order-statuses).
  pub status: OrderStatus,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub shipping_details: Option<OrderShippingDetails>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub total_details: Option<OrderTotalDetails>,
}

impl Order {
  /// Creates a new order object.
  pub fn create(client: &Client, params: CreateOrder<'_>) -> Response<Order> {
    client.post_form("/orders", &params)
  }

  /// Retrieves the details of an existing order.
  ///
  /// Supply the unique order ID from either an order creation request or the order list, and Stripe will return the corresponding order information.
  pub fn retrieve(client: &Client, id: &OrderId, expand: &[&str]) -> Response<Order> {
    client.get_query(&format!("/orders/{}", id), &Expand { expand })
  }

  /// Updates the specific order by setting the values of the parameters passed.
  ///
  /// Any parameters not provided will be left unchanged.
  pub fn update(client: &Client, id: &OrderId, params: UpdateOrder<'_>) -> Response<Order> {
    client.post_form(&format!("/orders/{}", id), &params)
  }
}

impl Object for Order {
  type Id = OrderId;
  fn id(&self) -> Self::Id {
    self.id.clone()
  }
  fn object(&self) -> &'static str {
    "order"
  }
}

/// The parameters for `Order::create`.
#[derive(Clone, Debug, Serialize)]
pub struct CreateOrder<'a> {}

impl<'a> CreateOrder<'a> {
  pub fn new(currency: Currency) -> Self {
    CreateOrder {}
  }
}

/// The parameters for `Order::update`.
#[derive(Clone, Debug, Serialize, Default)]
pub struct UpdateOrder<'a> {}

impl<'a> UpdateOrder<'a> {
  pub fn new() -> Self {
    UpdateOrder {}
  }
}
