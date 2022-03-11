use serde::{Deserialize, Serialize};
use stripe::{
  params as stripe_params, Address, Application, BillingDetails, Client, CouponId,
  CreatePriceRecurringInterval, Currency, Customer, CustomerId, Discount, DiscountId, Expandable,
  List, Metadata, Object, ParseIdError, PaymentIntent, PaymentIntentPaymentMethodOptions, Price,
  PriceId, PriceTaxBehavior, ProductId, PromotionCodeId, Response, TaxIdType, TaxRate, TaxRateId,
  Timestamp, TransferData,
};

def_id!(OrderId, "order_");

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
pub struct OrderTotalDetailsBreakdownDiscount {
  pub amount: i64,

  pub discount: Discount,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct OrderTax {
  pub amount: i64,

  pub rate: TaxRate,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct OrderTotalDetailsBreakdown {
  /// The aggregated line item discounts.
  pub discounts: Vec<OrderTotalDetailsBreakdownDiscount>,

  /// The aggregated line item tax amounts by rate.
  pub taxes: Vec<OrderTax>,
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
pub struct OrderTaxIds {
  /// The type of the tax ID, one of `eu_vat`, `br_cnpj`, `br_cpf`, `gb_vat`, `nz_gst`, `au_abn`, `au_arn`, `in_gst`, `no_vat`, `za_vat`, `ch_vat`, `mx_rfc`, `sg_uen`, `ru_inn`, `ru_kpp`, `ca_bn`, `hk_br`, `es_cif`, `tw_vat`, `th_vat`, `jp_cn`, `jp_rn`, `li_uid`, `my_itn`, `us_ein`, `kr_brn`, `ca_qst`, `ca_gst_hst`, `ca_pst_bc`, `ca_pst_mb`, `ca_pst_sk`, `my_sst`, `sg_gst`, `ae_trn`, `cl_tin`, `sa_vat`, `id_npwp`, `my_frp`, `il_vat`, `ge_vat`, `ua_vat`, `is_vat`, or `unknown`.
  #[serde(rename = "type")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub type_: Option<TaxIdType>,

  /// The value of the tax ID.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub value: Option<String>,
}

/// An enum representing the possible values of an `CreateInvoicePaymentSettings`'s `payment_method_types` field.
#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OrderPaymentSettingsPaymentMethodType {
  Card,
  Ideal,
  SepaDebit,
  Eps,
  WechatPay,
  Oxxo,
  Bancontact,
  Alipay,
  P24,
  BacsDebit,
  Giropay,
  Sofort,
  AuBecsDebit,
  Fpx,
  Klarna,
  Paypal,
  AcssDebit,
  Grabpay,
  AfterpayClearpay,
}

impl OrderPaymentSettingsPaymentMethodType {
  pub fn as_str(self) -> &'static str {
    match self {
      OrderPaymentSettingsPaymentMethodType::Card => "card",
      OrderPaymentSettingsPaymentMethodType::Ideal => "ideal",
      OrderPaymentSettingsPaymentMethodType::SepaDebit => "sepa_debit",
      OrderPaymentSettingsPaymentMethodType::Eps => "eps",
      OrderPaymentSettingsPaymentMethodType::WechatPay => "wechat_pay",
      OrderPaymentSettingsPaymentMethodType::Oxxo => "oxxo",
      OrderPaymentSettingsPaymentMethodType::Bancontact => "bancontact",
      OrderPaymentSettingsPaymentMethodType::Alipay => "alipay",
      OrderPaymentSettingsPaymentMethodType::P24 => "p24",
      OrderPaymentSettingsPaymentMethodType::BacsDebit => "bacs_debit",
      OrderPaymentSettingsPaymentMethodType::Giropay => "giropay",
      OrderPaymentSettingsPaymentMethodType::Sofort => "sofort",
      OrderPaymentSettingsPaymentMethodType::AuBecsDebit => "au_becs_debit",
      OrderPaymentSettingsPaymentMethodType::Fpx => "fpx",
      OrderPaymentSettingsPaymentMethodType::Klarna => "klarna",
      OrderPaymentSettingsPaymentMethodType::Paypal => "paypal",
      OrderPaymentSettingsPaymentMethodType::AcssDebit => "acss_debit",
      OrderPaymentSettingsPaymentMethodType::Grabpay => "grabpay",
      OrderPaymentSettingsPaymentMethodType::AfterpayClearpay => "afterpay_clearpay",
    }
  }
}

impl AsRef<str> for OrderPaymentSettingsPaymentMethodType {
  fn as_ref(&self) -> &str {
    self.as_str()
  }
}

impl std::fmt::Display for OrderPaymentSettingsPaymentMethodType {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    self.as_str().fmt(f)
  }
}
impl std::default::Default for OrderPaymentSettingsPaymentMethodType {
  fn default() -> Self {
    Self::Card
  }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct OrderPaymentSettings {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub application_fee_amount: Option<i64>,

  /// Payment-method-specific configuration to provide to the invoice’s PaymentIntent.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub payment_method_options: Option<PaymentIntentPaymentMethodOptions>,

  /// The list of payment method types (e.g.
  ///
  /// card) to provide to the invoice’s PaymentIntent.
  /// If not set, Stripe attempts to automatically determine the types to use by looking at the invoice’s default payment method, the subscription’s default payment method, the customer’s default payment method, and your [invoice template settings](https://dashboard.stripe.com/settings/billing/invoice).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub payment_method_types: Option<Vec<OrderPaymentSettingsPaymentMethodType>>,

  /// The URL to redirect the customer to after they authenticate their payment.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub return_url: Option<String>,

  pub statement_descriptor: Option<String>,

  pub statement_descriptor_suffix: Option<String>,

  pub transfer_data: Option<TransferData>,
}

impl OrderPaymentSettings {
  pub fn new() -> Self {
    Self {
      application_fee_amount: Default::default(),
      payment_method_options: Default::default(),
      payment_method_types: Default::default(),
      return_url: Default::default(),
      statement_descriptor: Default::default(),
      statement_descriptor_suffix: Default::default(),
      transfer_data: Default::default(),
    }
  }
}

/// An enum representing the possible values of an `PaymentIntent`'s `status` field.
#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OrderPaymentStatus {
  NotRequired,
  RequiresPaymentMethod,
  RequiresConfirmation,
  RequiresAction,
  Processing,
  Complete,
  RequiresCapture,
  Canceled,
}

impl OrderPaymentStatus {
  pub fn as_str(self) -> &'static str {
    match self {
      OrderPaymentStatus::NotRequired => "not_required",
      OrderPaymentStatus::RequiresPaymentMethod => "requires_payment_method",
      OrderPaymentStatus::RequiresConfirmation => "requires_confirmation",
      OrderPaymentStatus::RequiresAction => "requires_action",
      OrderPaymentStatus::Processing => "processing",
      OrderPaymentStatus::Complete => "complete",
      OrderPaymentStatus::RequiresCapture => "requires_capture",
      OrderPaymentStatus::Canceled => "canceled",
    }
  }
}

impl AsRef<str> for OrderPaymentStatus {
  fn as_ref(&self) -> &str {
    self.as_str()
  }
}

impl std::fmt::Display for OrderPaymentStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    self.as_str().fmt(f)
  }
}
impl std::default::Default for OrderPaymentStatus {
  fn default() -> Self {
    Self::NotRequired
  }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct OrderPayment {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub payment_intent: Option<Expandable<PaymentIntent>>,

  pub settings: OrderPaymentSettings,

  pub status: Option<OrderPaymentStatus>,
}

/// The resource representing a Stripe "OrderItem".
///
/// For more details see <https://stripe.com/docs/api/order_items/object>
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct OrderLineItem {
  pub amount_discount: i64,

  pub amount_subtotal: i64,

  pub amount_tax: i64,

  pub amount_total: i64,

  pub currency: Currency,

  pub description: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub discounts: Option<Vec<Discount>>,

  pub price: Price,

  pub quantity: i64,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub taxes: Option<Vec<OrderTax>>,
}

impl Object for OrderLineItem {
  type Id = ();
  fn id(&self) -> Self::Id {}
  fn object(&self) -> &'static str {
    "item"
  }
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

  pub automatic_tax: OrderAutomaticTax,

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

  pub discounts: Vec<Expandable<Discount>>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub ip_address: Option<String>,

  /// Has the value `true` if the object exists in live mode or the value `false` if the object exists in test mode.
  pub livemode: bool,

  /// List of items constituting the order.
  ///
  /// An order can have up to 25 items.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub line_items: Option<List<Expandable<OrderLineItem>>>,

  /// Set of [key-value pairs](https://stripe.com/docs/api/metadata) that you can attach to an object.
  ///
  /// This can be useful for storing additional information about the object in a structured format.
  #[serde(default)]
  pub metadata: Metadata,

  pub payment: OrderPayment,

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

  pub fn submit(client: &Client, id: &OrderId, params: SubmitOrder) -> Response<Order> {
    client.post_form(&format!("/orders/{}/submit", id), &params)
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

#[derive(Clone, Debug, Serialize)]
pub struct CreateOrderLineItemDiscount {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub coupon: Option<CouponId>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub discount: Option<DiscountId>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CreateOrderLineItemPriceDataRecurring {
  pub interval: CreatePriceRecurringInterval,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub interval_count: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CreateOrderLineItemPriceData {
  pub currency: Currency,

  pub product: ProductId,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub unit_amount_decimal: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub metadata: Option<Metadata>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub recurring: Option<CreateOrderLineItemPriceDataRecurring>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub tax_behavior: Option<PriceTaxBehavior>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub unit_amount: Option<i64>,
}

impl CreateOrderLineItemPriceData {
  pub fn new(currency: Currency, product: ProductId) -> Self {
    CreateOrderLineItemPriceData {
      currency,
      product,
      unit_amount_decimal: Default::default(),
      metadata: Default::default(),
      recurring: Default::default(),
      tax_behavior: Default::default(),
      unit_amount: Default::default(),
    }
  }
}

#[derive(Clone, Debug, Serialize)]
pub struct CreateOrderLineItem {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub discounts: Option<Vec<CreateOrderLineItemDiscount>>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub price: Option<PriceId>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub price_data: Option<CreateOrderLineItemPriceData>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub product: Option<ProductId>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub quantity: Option<i64>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub tax_rates: Option<Vec<TaxRateId>>,
}

impl CreateOrderLineItem {
  pub fn new() -> Self {
    CreateOrderLineItem {
      description: Default::default(),
      discounts: Default::default(),
      price: Default::default(),
      price_data: Default::default(),
      product: Default::default(),
      quantity: Default::default(),
      tax_rates: Default::default(),
    }
  }
}

#[derive(Clone, Debug, Serialize)]
pub struct CreateOrderPayment {
  pub settings: OrderPaymentSettings,
}

#[derive(Clone, Debug, Serialize)]
pub struct CreateOrderAutomaticTax {
  pub enabled: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct CreateOrderDiscount {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub coupon: Option<CouponId>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub discount: Option<DiscountId>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub promotion_code: Option<PromotionCodeId>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CreateOrderTaxDetails {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tax_exempt: Option<OrderTaxExempt>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub tax_ids: Option<Vec<OrderTaxIds>>,
}

/// The parameters for `Order::create`.
#[derive(Clone, Debug, Serialize)]
pub struct CreateOrder<'a> {
  pub currency: Currency,

  #[serde(skip_serializing_if = "Expand::is_empty")]
  pub expand: &'a [&'a str],

  pub line_items: Vec<CreateOrderLineItem>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub customer: Option<CustomerId>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub metadata: Option<Metadata>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub payment: Option<CreateOrderPayment>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub automatic_tax: Option<CreateOrderAutomaticTax>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub billing_details: Option<BillingDetails>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub discounts: Option<Vec<CreateOrderDiscount>>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub ip_address: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub shipping_details: Option<OrderShippingDetails>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub tax_details: Option<CreateOrderTaxDetails>,
}

impl<'a> CreateOrder<'a> {
  pub fn new(currency: Currency, line_items: Vec<CreateOrderLineItem>) -> Self {
    CreateOrder {
      currency,
      line_items,
      expand: Default::default(),
      customer: Default::default(),
      description: Default::default(),
      metadata: Default::default(),
      payment: Default::default(),
      automatic_tax: Default::default(),
      billing_details: Default::default(),
      discounts: Default::default(),
      ip_address: Default::default(),
      shipping_details: Default::default(),
      tax_details: Default::default(),
    }
  }
}

/// The parameters for `Order::update`.
#[derive(Clone, Debug, Serialize, Default)]
pub struct UpdateOrder<'a> {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub currency: Option<Currency>,

  #[serde(skip_serializing_if = "Expand::is_empty")]
  pub expand: &'a [&'a str],

  #[serde(skip_serializing_if = "Option::is_none")]
  pub line_items: Option<Vec<CreateOrderLineItem>>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub customer: Option<CustomerId>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub metadata: Option<Metadata>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub payment: Option<CreateOrderPayment>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub automatic_tax: Option<CreateOrderAutomaticTax>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub billing_details: Option<BillingDetails>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub discounts: Option<Vec<CreateOrderDiscount>>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub ip_address: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub shipping_details: Option<OrderShippingDetails>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub tax_details: Option<CreateOrderTaxDetails>,
}

impl<'a> UpdateOrder<'a> {
  pub fn new() -> Self {
    UpdateOrder {
      currency: Default::default(),
      line_items: Default::default(),
      expand: Default::default(),
      customer: Default::default(),
      description: Default::default(),
      metadata: Default::default(),
      payment: Default::default(),
      automatic_tax: Default::default(),
      billing_details: Default::default(),
      discounts: Default::default(),
      ip_address: Default::default(),
      shipping_details: Default::default(),
      tax_details: Default::default(),
    }
  }
}

#[derive(Clone, Debug, Serialize)]
pub struct SubmitOrder<'a> {
  pub expected_total: i64,

  pub expand: &'a [&'a str],
}

impl<'a> SubmitOrder<'a> {
  pub fn new(expected_total: i64) -> Self {
    SubmitOrder {
      expected_total,
      expand: Default::default(),
    }
  }
}
