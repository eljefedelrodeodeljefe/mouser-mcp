//! Mouser API data types, generated from Mouser's Swagger documents
//! (<https://api.mouser.com/api/docs/V1>, <https://api.mouser.com/api/docs/V2>) by
//! `tools/gen_models.py`. Do not edit by hand: regenerate.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct AdditionalFee {
    #[serde(rename = "amount", default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<f64>,
    #[serde(
        rename = "extendedAmount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub extended_amount: Option<f64>,
    #[serde(rename = "code", default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct AddressModel {
    #[serde(
        rename = "CountryCode",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub country_code: Option<String>,
    #[serde(
        rename = "AttentionLine",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub attention_line: Option<String>,
    #[serde(
        rename = "CompanyName",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub company_name: Option<String>,
    #[serde(
        rename = "AddressOne",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub address_one: Option<String>,
    #[serde(
        rename = "AddressTwo",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub address_two: Option<String>,
    #[serde(rename = "City", default, skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(
        rename = "StateOrProvince",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub state_or_province: Option<String>,
    #[serde(
        rename = "PostalCode",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub postal_code: Option<String>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct AlternatePackaging {
    #[serde(rename = "APMfrPN", default, skip_serializing_if = "Option::is_none")]
    pub ap_mfr_pn: Option<String>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct ApiError {
    #[serde(rename = "ErrorType", default, skip_serializing_if = "Option::is_none")]
    pub error_type: Option<String>,
    #[serde(rename = "Errors", default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<ErrorEntity>>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct AvailabilityOnOrderObject {
    #[serde(rename = "Quantity", default, skip_serializing_if = "Option::is_none")]
    pub quantity: Option<i64>,
    #[serde(rename = "Date", default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct CartItemRequest {
    #[serde(rename = "MouserPartNumber")]
    pub mouser_part_number: String,
    #[serde(rename = "Quantity")]
    pub quantity: i64,
    #[serde(
        rename = "CustomerPartNumber",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub customer_part_number: Option<String>,
    #[serde(
        rename = "PackagingChoice",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub packaging_choice: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct CartItemRequestRoot {
    #[serde(rename = "CartKey", default, skip_serializing_if = "Option::is_none")]
    pub cart_key: Option<String>,
    #[serde(
        rename = "MouserPaysCustomsAndDuties",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub mouser_pays_customs_and_duties: Option<bool>,
    #[serde(rename = "CartItems", default, skip_serializing_if = "Option::is_none")]
    pub cart_items: Option<Vec<CartItemRequest>>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct CartResponseRoot {
    #[serde(rename = "Errors", default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<ErrorEntity>>,
    #[serde(rename = "CartKey", default, skip_serializing_if = "Option::is_none")]
    pub cart_key: Option<String>,
    #[serde(
        rename = "CurrencyCode",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub currency_code: Option<String>,
    #[serde(rename = "CartItems", default, skip_serializing_if = "Option::is_none")]
    pub cart_items: Option<Vec<OrderLine>>,
    #[serde(
        rename = "TotalItemCount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub total_item_count: Option<i64>,
    #[serde(
        rename = "additionalFeesTotal",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub additional_fees_total: Option<f64>,
    #[serde(
        rename = "MerchandiseTotal",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub merchandise_total: Option<f64>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct CountryResponse {
    #[serde(
        rename = "CountryName",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub country_name: Option<String>,
    #[serde(
        rename = "CountryCode",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub country_code: Option<String>,
    #[serde(rename = "States", default, skip_serializing_if = "Option::is_none")]
    pub states: Option<Vec<StateResponse>>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct CountryRoot {
    #[serde(rename = "Countries", default, skip_serializing_if = "Option::is_none")]
    pub countries: Option<Vec<CountryResponse>>,
    #[serde(rename = "Errors", default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<ErrorEntity>>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct DeliveryModel {
    #[serde(
        rename = "ShippingMethodName",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub shipping_method_name: Option<String>,
    #[serde(
        rename = "BackOrderShippingMethodName",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub back_order_shipping_method_name: Option<String>,
    #[serde(
        rename = "TrackingDetails",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub tracking_details: Option<Vec<TrackingModel>>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct ErrorEntity {
    #[serde(rename = "Id", default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(rename = "Code", default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(rename = "Message", default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(
        rename = "ResourceKey",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub resource_key: Option<String>,
    #[serde(
        rename = "ResourceFormatString",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub resource_format_string: Option<String>,
    #[serde(
        rename = "ResourceFormatString2",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub resource_format_string2: Option<String>,
    #[serde(
        rename = "PropertyName",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub property_name: Option<String>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct FreightAccount {
    #[serde(rename = "Number", default, skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,
    #[serde(rename = "Type", default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct KeyValuePairOfStringAndInt32 {
    #[serde(rename = "key", default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(rename = "value", default, skip_serializing_if = "Option::is_none")]
    pub value: Option<i64>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct MouserManufacturerName {
    #[serde(
        rename = "ManufacturerName",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub manufacturer_name: Option<String>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct MouserManufacturerNameList {
    #[serde(rename = "Count", default, skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,
    #[serde(
        rename = "ManufacturerList",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub manufacturer_list: Option<Vec<MouserManufacturerName>>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct MouserManufacturersNameRoot {
    #[serde(
        rename = "MouserManufacturerList",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub mouser_manufacturer_list: Option<MouserManufacturerNameList>,
    #[serde(rename = "Errors", default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<ErrorEntity>>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct MouserPart {
    #[serde(
        rename = "Availability",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub availability: Option<String>,
    #[serde(
        rename = "DataSheetUrl",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub data_sheet_url: Option<String>,
    #[serde(
        rename = "Description",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,
    #[serde(
        rename = "FactoryStock",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub factory_stock: Option<String>,
    #[serde(rename = "ImagePath", default, skip_serializing_if = "Option::is_none")]
    pub image_path: Option<String>,
    #[serde(rename = "Category", default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(rename = "LeadTime", default, skip_serializing_if = "Option::is_none")]
    pub lead_time: Option<String>,
    #[serde(
        rename = "LifecycleStatus",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub lifecycle_status: Option<String>,
    #[serde(
        rename = "Manufacturer",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub manufacturer: Option<String>,
    #[serde(
        rename = "ManufacturerPartNumber",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub manufacturer_part_number: Option<String>,
    #[serde(rename = "Min", default, skip_serializing_if = "Option::is_none")]
    pub min: Option<String>,
    #[serde(rename = "Mult", default, skip_serializing_if = "Option::is_none")]
    pub mult: Option<String>,
    #[serde(
        rename = "MouserPartNumber",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub mouser_part_number: Option<String>,
    #[serde(
        rename = "ProductAttributes",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub product_attributes: Option<Vec<ProductAttribute>>,
    #[serde(
        rename = "PriceBreaks",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub price_breaks: Option<Vec<Pricebreak>>,
    #[serde(
        rename = "AlternatePackagings",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub alternate_packagings: Option<Vec<AlternatePackaging>>,
    #[serde(
        rename = "ProductDetailUrl",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub product_detail_url: Option<String>,
    #[serde(rename = "Reeling", default, skip_serializing_if = "Option::is_none")]
    pub reeling: Option<bool>,
    #[serde(
        rename = "ROHSStatus",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub rohs_status: Option<String>,
    #[serde(
        rename = "REACH-SVHC",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub reach_svhc: Option<Vec<String>>,
    #[serde(
        rename = "SuggestedReplacement",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub suggested_replacement: Option<String>,
    #[serde(
        rename = "MultiSimBlue",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub multi_sim_blue: Option<i64>,
    #[serde(
        rename = "UnitWeightKg",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub unit_weight_kg: Option<UnitWeightKg>,
    #[serde(
        rename = "StandardCost",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub standard_cost: Option<StandardCost>,
    #[serde(
        rename = "IsDiscontinued",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub is_discontinued: Option<String>,
    #[serde(rename = "RTM", default, skip_serializing_if = "Option::is_none")]
    pub rtm: Option<String>,
    #[serde(
        rename = "MouserProductCategory",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub mouser_product_category: Option<String>,
    #[serde(rename = "IPCCode", default, skip_serializing_if = "Option::is_none")]
    pub ipc_code: Option<String>,
    #[serde(rename = "SField", default, skip_serializing_if = "Option::is_none")]
    pub s_field: Option<String>,
    #[serde(rename = "VNum", default, skip_serializing_if = "Option::is_none")]
    pub v_num: Option<String>,
    #[serde(
        rename = "ActualMfrName",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub actual_mfr_name: Option<String>,
    #[serde(
        rename = "AvailableOnOrder",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub available_on_order: Option<String>,
    #[serde(
        rename = "AvailabilityInStock",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub availability_in_stock: Option<String>,
    #[serde(
        rename = "AvailabilityOnOrder",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub availability_on_order: Option<Vec<AvailabilityOnOrderObject>>,
    #[serde(
        rename = "InfoMessages",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub info_messages: Option<Vec<String>>,
    #[serde(
        rename = "SalesMaximumOrderQty",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub sales_maximum_order_qty: Option<String>,
    #[serde(
        rename = "SurchargeMessages",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub surcharge_messages: Option<Vec<SurchargeMessages>>,
    #[serde(
        rename = "RestrictionMessage",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub restriction_message: Option<String>,
    #[serde(rename = "PID", default, skip_serializing_if = "Option::is_none")]
    pub pid: Option<String>,
    #[serde(
        rename = "ProductCompliance",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub product_compliance: Option<Vec<ProductCompliance>>,
    #[serde(
        rename = "TradeCompliance",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub trade_compliance: Option<Vec<TradeCompliance>>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct OHAdditionalFee {
    #[serde(rename = "amount", default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,
    #[serde(
        rename = "extendedAmount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub extended_amount: Option<String>,
    #[serde(rename = "code", default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct Order {
    #[serde(
        rename = "OrderLines",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub order_lines: Option<Vec<OrderLine>>,
    #[serde(
        rename = "CurrencyCode",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub currency_code: Option<String>,
    #[serde(
        rename = "MerchandiseTotal",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub merchandise_total: Option<f64>,
    #[serde(
        rename = "additionalFeesTotal",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub additional_fees_total: Option<f64>,
    #[serde(
        rename = "OrderTotal",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub order_total: Option<f64>,
    #[serde(rename = "OrderType", default, skip_serializing_if = "Option::is_none")]
    pub order_type: Option<String>,
    #[serde(rename = "CartGUID", default, skip_serializing_if = "Option::is_none")]
    pub cart_guid: Option<String>,
    #[serde(
        rename = "BillingAddress",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub billing_address: Option<OrderAddressType>,
    #[serde(
        rename = "ShippingAddress",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub shipping_address: Option<OrderAddressType>,
    #[serde(
        rename = "ShippingMethod",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub shipping_method: Option<OrderShipping>,
    #[serde(
        rename = "PaymentMethod",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub payment_method: Option<OrderPayment>,
    #[serde(rename = "TaxAmount", default, skip_serializing_if = "Option::is_none")]
    pub tax_amount: Option<f64>,
    #[serde(rename = "OrderID", default, skip_serializing_if = "Option::is_none")]
    pub order_id: Option<String>,
    #[serde(
        rename = "TaxCertificateId",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub tax_certificate_id: Option<String>,
    #[serde(rename = "IECCode", default, skip_serializing_if = "Option::is_none")]
    pub iec_code: Option<String>,
    #[serde(
        rename = "LanguageCode",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub language_code: Option<String>,
    #[serde(rename = "Errors", default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<ErrorEntity>>,
    #[serde(
        rename = "SubmitOrder",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub submit_order: Option<bool>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct OrderAddress {
    /// One of: Residential, Commercial, PostOfficeBox, APOFPO.
    #[serde(
        rename = "AddressLocationTypeID",
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::lenient::enum_text"
    )]
    pub address_location_type_id: Option<String>,
    #[serde(rename = "CountryCode")]
    pub country_code: String,
    #[serde(rename = "FirstName")]
    pub first_name: String,
    #[serde(rename = "LastName")]
    pub last_name: String,
    #[serde(
        rename = "AttentionLine",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub attention_line: Option<String>,
    #[serde(rename = "Company", default, skip_serializing_if = "Option::is_none")]
    pub company: Option<String>,
    #[serde(rename = "AddressOne")]
    pub address_one: String,
    #[serde(
        rename = "AddressTwo",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub address_two: Option<String>,
    #[serde(rename = "City")]
    pub city: String,
    /// Required for certain countries.
    #[serde(
        rename = "StateOrProvince",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub state_or_province: Option<String>,
    /// Required for certain countries. Max length is dependent upon country. China for example has
    /// a max length of 6. Default max length is 10.
    #[serde(
        rename = "PostalCode",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub postal_code: Option<String>,
    #[serde(rename = "PhoneNumber")]
    pub phone_number: String,
    #[serde(
        rename = "PhoneExtension",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub phone_extension: Option<String>,
    /// Only required for billing addresses
    #[serde(
        rename = "EmailAddress",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub email_address: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct OrderAddressType {
    #[serde(rename = "Error", default, skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
    /// One of: Residential, Commercial, PostOfficeBox, APOFPO.
    #[serde(
        rename = "AddressLocationTypeID",
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::lenient::enum_text"
    )]
    pub address_location_type_id: Option<String>,
    #[serde(
        rename = "CountryCode",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub country_code: Option<String>,
    #[serde(rename = "FirstName", default, skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(rename = "LastName", default, skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(
        rename = "AttentionLine",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub attention_line: Option<String>,
    #[serde(rename = "Company", default, skip_serializing_if = "Option::is_none")]
    pub company: Option<String>,
    #[serde(
        rename = "AddressOne",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub address_one: Option<String>,
    #[serde(
        rename = "AddressTwo",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub address_two: Option<String>,
    #[serde(rename = "City", default, skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    /// Required for certain countries.
    #[serde(
        rename = "StateOrProvince",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub state_or_province: Option<String>,
    /// Required for certain countries. Max length is dependent upon country. China for example has
    /// a max length of 6. Default max length is 10.
    #[serde(
        rename = "PostalCode",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub postal_code: Option<String>,
    #[serde(
        rename = "PhoneNumber",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub phone_number: Option<String>,
    #[serde(
        rename = "PhoneExtension",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub phone_extension: Option<String>,
    /// Only required for billing addresses
    #[serde(
        rename = "EmailAddress",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub email_address: Option<String>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct OrderCurrency {
    #[serde(
        rename = "CurrencyCode",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub currency_code: Option<String>,
    #[serde(
        rename = "CurrencyName",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub currency_name: Option<String>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct OrderCurrencyRoot {
    #[serde(rename = "Errors", default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<ErrorEntity>>,
    #[serde(
        rename = "Currencies",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub currencies: Option<Vec<OrderCurrency>>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct OrderDetail {
    #[serde(
        rename = "OrderLines",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub order_lines: Option<Vec<OrderLineItem>>,
    #[serde(
        rename = "SalesOrderId",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub sales_order_id: Option<String>,
    #[serde(
        rename = "WebOrderId",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub web_order_id: Option<i64>,
    #[serde(
        rename = "OrderStatus",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub order_status: Option<i64>,
    #[serde(
        rename = "OrderStatusName",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub order_status_name: Option<String>,
    #[serde(rename = "OrderDate", default, skip_serializing_if = "Option::is_none")]
    pub order_date: Option<String>,
    #[serde(
        rename = "BillingAddress",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub billing_address: Option<AddressModel>,
    #[serde(
        rename = "ShippingAddress",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub shipping_address: Option<AddressModel>,
    #[serde(
        rename = "PaymentDetail",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub payment_detail: Option<PaymentModel>,
    #[serde(
        rename = "DeliveryDetail",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub delivery_detail: Option<DeliveryModel>,
    #[serde(
        rename = "CurrencyCode",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub currency_code: Option<String>,
    #[serde(
        rename = "IsScheduled",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub is_scheduled: Option<bool>,
    #[serde(
        rename = "IsPendingOrder",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub is_pending_order: Option<bool>,
    #[serde(rename = "BuyerName", default, skip_serializing_if = "Option::is_none")]
    pub buyer_name: Option<String>,
    #[serde(
        rename = "SummaryDetail",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub summary_detail: Option<OrderDetailSummary>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct OrderDetailSummary {
    #[serde(
        rename = "MerchandiseTotal",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub merchandise_total: Option<String>,
    #[serde(
        rename = "OrderTotal",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub order_total: Option<String>,
    #[serde(
        rename = "AdditionalFeesTotal",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub additional_fees_total: Option<String>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct OrderEInvoicingDefinition {
    #[serde(rename = "LabelKey", default, skip_serializing_if = "Option::is_none")]
    pub label_key: Option<String>,
    #[serde(
        rename = "EnglishLabel",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub english_label: Option<String>,
    #[serde(
        rename = "LocalLanguageLabel",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub local_language_label: Option<String>,
    #[serde(
        rename = "RegularExpression",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub regular_expression: Option<String>,
    #[serde(rename = "Required", default, skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(rename = "FieldType", default, skip_serializing_if = "Option::is_none")]
    pub field_type: Option<String>,
    #[serde(
        rename = "Description",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,
    #[serde(
        rename = "SelectValues",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub select_values: Option<Vec<OrderEInvoicingSelectValue>>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct OrderEInvoicingSelectValue {
    #[serde(rename = "LabelKey", default, skip_serializing_if = "Option::is_none")]
    pub label_key: Option<String>,
    #[serde(
        rename = "SelectValue",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub select_value: Option<String>,
    #[serde(
        rename = "Description",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct OrderHistoryBaseObject {
    #[serde(
        rename = "DateCreated",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub date_created: Option<String>,
    #[serde(
        rename = "SalesOrderNumber",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub sales_order_number: Option<String>,
    #[serde(
        rename = "WebOrderNumber",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub web_order_number: Option<String>,
    #[serde(rename = "PoNumber", default, skip_serializing_if = "Option::is_none")]
    pub po_number: Option<String>,
    #[serde(rename = "BuyerName", default, skip_serializing_if = "Option::is_none")]
    pub buyer_name: Option<String>,
    #[serde(
        rename = "OrderStatusDisplay",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub order_status_display: Option<String>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct OrderHistoryResponseRoot {
    #[serde(rename = "Errors", default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<ErrorEntity>>,
    #[serde(
        rename = "NumberOfOrders",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub number_of_orders: Option<i64>,
    #[serde(
        rename = "OrderHistoryItems",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub order_history_items: Option<Vec<OrderHistoryBaseObject>>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct OrderInializeRequestRoot {
    #[serde(
        rename = "OrderInitialize",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub order_initialize: Option<OrderInitializeRequest>,
}

/// The billing address associated with this account will be used. Shipping Address is optional.
/// If a shipping address is not provided, the default shipping address associated with this
/// account will be used.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct OrderInitializeRequest {
    /// Comments here won't show up in swagger docs UI page for some reason
    #[serde(
        rename = "ShippingAddress",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub shipping_address: Option<OrderAddress>,
    /// Optional. If not provided, the default currency code based on the billing/shipping address
    /// associated with the account will be used to determine currency code.
    #[serde(
        rename = "CurrencyCode",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub currency_code: Option<String>,
    /// Optional. However, if a cart key is provided, more accurate shipping methods will be
    /// returned, plus shipping rates.
    #[serde(rename = "CartKey", default, skip_serializing_if = "Option::is_none")]
    pub cart_key: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct OrderInitializeResponse {
    #[serde(
        rename = "CurrencyCode",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub currency_code: Option<String>,
    #[serde(
        rename = "BillingAddress",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub billing_address: Option<OrderAddressType>,
    #[serde(
        rename = "ShippingAddress",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub shipping_address: Option<OrderAddressType>,
    #[serde(rename = "Shipping", default, skip_serializing_if = "Option::is_none")]
    pub shipping: Option<ShippingRoot>,
    #[serde(rename = "IECCode", default, skip_serializing_if = "Option::is_none")]
    pub iec_code: Option<String>,
    #[serde(rename = "Payment", default, skip_serializing_if = "Option::is_none")]
    pub payment: Option<PaymentRoot>,
    #[serde(rename = "Languages", default, skip_serializing_if = "Option::is_none")]
    pub languages: Option<Vec<PreferredLanguage>>,
    #[serde(
        rename = "EInvoicing",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub e_invoicing: Option<Vec<OrderEInvoicingDefinition>>,
    #[serde(rename = "Errors", default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<ErrorEntity>>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct OrderInitializeResponseRoot {
    #[serde(
        rename = "OrderInitialize",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub order_initialize: Option<OrderInitializeResponse>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct OrderLine {
    #[serde(rename = "Errors", default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<ErrorEntity>>,
    #[serde(rename = "MouserATS", default, skip_serializing_if = "Option::is_none")]
    pub mouser_ats: Option<i64>,
    #[serde(rename = "Quantity", default, skip_serializing_if = "Option::is_none")]
    pub quantity: Option<i64>,
    #[serde(
        rename = "PartsPerReel",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub parts_per_reel: Option<i64>,
    #[serde(
        rename = "PackagingChoice",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub packaging_choice: Option<String>,
    #[serde(
        rename = "ScheduledReleases",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub scheduled_releases: Option<Vec<ScheduleRelease>>,
    #[serde(
        rename = "InfoMessages",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub info_messages: Option<Vec<String>>,
    #[serde(
        rename = "MouserPartNumber",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub mouser_part_number: Option<String>,
    #[serde(
        rename = "MfrPartNumber",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub mfr_part_number: Option<String>,
    #[serde(
        rename = "Description",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,
    #[serde(
        rename = "CartItemCustPartNumber",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub cart_item_cust_part_number: Option<String>,
    #[serde(rename = "UnitPrice", default, skip_serializing_if = "Option::is_none")]
    pub unit_price: Option<f64>,
    #[serde(
        rename = "ExtendedPrice",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub extended_price: Option<f64>,
    #[serde(rename = "LifeCycle", default, skip_serializing_if = "Option::is_none")]
    pub life_cycle: Option<String>,
    #[serde(
        rename = "Manufacturer",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub manufacturer: Option<String>,
    #[serde(
        rename = "SalesMultipleQty",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub sales_multiple_qty: Option<i64>,
    #[serde(
        rename = "SalesMinimumOrderQty",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub sales_minimum_order_qty: Option<i64>,
    #[serde(
        rename = "SalesMaximumOrderQty",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub sales_maximum_order_qty: Option<i64>,
    #[serde(
        rename = "additionalFees",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub additional_fees: Option<Vec<AdditionalFee>>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct OrderLineActivity {
    #[serde(
        rename = "InvoiceNumber",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub invoice_number: Option<i64>,
    #[serde(rename = "Date", default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct OrderLineItem {
    #[serde(rename = "Quantity", default, skip_serializing_if = "Option::is_none")]
    pub quantity: Option<i64>,
    #[serde(rename = "UnitPrice", default, skip_serializing_if = "Option::is_none")]
    pub unit_price: Option<f64>,
    #[serde(rename = "ExtPrice", default, skip_serializing_if = "Option::is_none")]
    pub ext_price: Option<f64>,
    #[serde(
        rename = "FormattedUnitPrice",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub formatted_unit_price: Option<String>,
    #[serde(
        rename = "FormattedExtendedPrice",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub formatted_extended_price: Option<String>,
    #[serde(
        rename = "AdditionalFees",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub additional_fees: Option<Vec<OHAdditionalFee>>,
    #[serde(
        rename = "ProductInfo",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub product_info: Option<OrderLineProduct>,
    #[serde(
        rename = "Activities",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub activities: Option<Vec<OrderLineActivity>>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct OrderLineProduct {
    #[serde(
        rename = "MouserPartNumber",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub mouser_part_number: Option<String>,
    #[serde(
        rename = "CustomerPartNumber",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub customer_part_number: Option<String>,
    #[serde(
        rename = "ManufacturerName",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub manufacturer_name: Option<String>,
    #[serde(
        rename = "ManufacturerPartNumber",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub manufacturer_part_number: Option<String>,
    #[serde(
        rename = "PartDescription",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub part_description: Option<String>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct OrderPayment {
    #[serde(rename = "Error", default, skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
    #[serde(rename = "PONumber", default, skip_serializing_if = "Option::is_none")]
    pub po_number: Option<String>,
    #[serde(rename = "Method", default, skip_serializing_if = "Option::is_none")]
    pub method: Option<i64>,
    #[serde(rename = "Name", default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(
        rename = "VatAccountNumber",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub vat_account_number: Option<String>,
    #[serde(
        rename = "VatInvoiceAddress",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub vat_invoice_address: Option<OrderAddressType>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct OrderRequestRootType {
    #[serde(rename = "Order", default, skip_serializing_if = "Option::is_none")]
    pub order: Option<OrderRequestType>,
}

/// The billing address associated with this account will be used. Shipping Address is optional.
/// If a shipping address is not provided, the default shipping address associated with this
/// account will be used.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct OrderRequestType {
    #[serde(
        rename = "ShippingAddress",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub shipping_address: Option<OrderAddress>,
    /// One of: Unspecified, Rush, Complete.
    #[serde(
        rename = "OrderType",
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::lenient::enum_text"
    )]
    pub order_type: Option<String>,
    #[serde(
        rename = "PrimaryShipping",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub primary_shipping: Option<ShippingMethod>,
    #[serde(
        rename = "SecondaryShipping",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub secondary_shipping: Option<ShippingMethod>,
    #[serde(rename = "IECCode", default, skip_serializing_if = "Option::is_none")]
    pub iec_code: Option<String>,
    #[serde(
        rename = "PrimaryFreightAccount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub primary_freight_account: Option<FreightAccount>,
    #[serde(
        rename = "SecondaryFreightAccount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub secondary_freight_account: Option<FreightAccount>,
    #[serde(rename = "Payment", default, skip_serializing_if = "Option::is_none")]
    pub payment: Option<PaymentTypeRequest>,
    #[serde(rename = "CurrencyCode")]
    pub currency_code: String,
    #[serde(rename = "CartKey")]
    pub cart_key: String,
    #[serde(
        rename = "LanguageCode",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub language_code: Option<String>,
    #[serde(
        rename = "SubmitOrder",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub submit_order: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct OrderRoot {
    #[serde(rename = "Order", default, skip_serializing_if = "Option::is_none")]
    pub order: Option<Order>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct OrderShipping {
    #[serde(rename = "Error", default, skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
    #[serde(
        rename = "PrimaryCode",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub primary_code: Option<i64>,
    #[serde(
        rename = "SecondaryCode",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub secondary_code: Option<i64>,
    #[serde(
        rename = "PrimaryMethod",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub primary_method: Option<String>,
    #[serde(
        rename = "SecondaryMethod",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub secondary_method: Option<String>,
    #[serde(
        rename = "PrimaryShippingRate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub primary_shipping_rate: Option<f64>,
    #[serde(
        rename = "SecondaryShippingRate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub secondary_shipping_rate: Option<f64>,
    #[serde(
        rename = "PrimaryFreightCollectAccount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub primary_freight_collect_account: Option<String>,
    #[serde(
        rename = "SecondaryFreightCollectAccount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub secondary_freight_collect_account: Option<String>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct PaymentModel {
    #[serde(rename = "PoNumber", default, skip_serializing_if = "Option::is_none")]
    pub po_number: Option<String>,
    #[serde(
        rename = "PaymentMethodName",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub payment_method_name: Option<String>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct PaymentRoot {
    #[serde(
        rename = "PaymentTypes",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub payment_types: Option<Vec<PaymentType>>,
    #[serde(
        rename = "TaxCertificates",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub tax_certificates: Option<Vec<TaxCertificates>>,
    #[serde(
        rename = "VatAccounts",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub vat_accounts: Option<Vec<String>>,
    #[serde(
        rename = "VatAccountDetails",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub vat_account_details: Option<Vec<VatAccount>>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct PaymentType {
    #[serde(rename = "Method", default, skip_serializing_if = "Option::is_none")]
    pub method: Option<i64>,
    #[serde(rename = "Name", default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(
        rename = "CreditCardLastFourDigits",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub credit_card_last_four_digits: Option<String>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct PaymentTypeRequest {
    #[serde(rename = "Method")]
    pub method: i64,
    /// Optional. For RMB orders only. If not provided, the billing address associated with the
    /// account will be used if there is not a VAT Invoice Address already on the account.
    #[serde(
        rename = "VatInvoiceAddress",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub vat_invoice_address: Option<OrderAddress>,
    #[serde(
        rename = "CreditCardLastFourDigits",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub credit_card_last_four_digits: Option<String>,
    #[serde(
        rename = "VatAccountNumber",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub vat_account_number: Option<String>,
    #[serde(
        rename = "TaxCertificateKey",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub tax_certificate_key: Option<String>,
    /// One of: None, Taxable, NonTaxable.
    #[serde(
        rename = "TaxStatus",
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::lenient::enum_text"
    )]
    pub tax_status: Option<String>,
    #[serde(rename = "PoNumber", default, skip_serializing_if = "Option::is_none")]
    pub po_number: Option<String>,
    /// Required only for RMB orders One of: None, GeneralWithoutTxID, General, Vat.
    #[serde(
        rename = "VatInvoiceType",
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::lenient::enum_text"
    )]
    pub vat_invoice_type: Option<String>,
    #[serde(rename = "BankName", default, skip_serializing_if = "Option::is_none")]
    pub bank_name: Option<String>,
    #[serde(
        rename = "BankAccountNumber",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub bank_account_number: Option<String>,
    /// Required only if shipping to Mexico. Do you have an RFC Number?
    #[serde(
        rename = "UseRfcNumber",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub use_rfc_number: Option<bool>,
    /// Is this number valid for EU intra-Community shipments?
    #[serde(
        rename = "VatNumberValidForEuShipments",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub vat_number_valid_for_eu_shipments: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct PreferredLanguage {
    #[serde(rename = "Code", default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(
        rename = "Description",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct Pricebreak {
    #[serde(rename = "Quantity", default, skip_serializing_if = "Option::is_none")]
    pub quantity: Option<i64>,
    #[serde(rename = "Price", default, skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    #[serde(rename = "Currency", default, skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct ProductAttribute {
    #[serde(
        rename = "AttributeName",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub attribute_name: Option<String>,
    #[serde(
        rename = "AttributeValue",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub attribute_value: Option<String>,
    #[serde(
        rename = "AttributeCost",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub attribute_cost: Option<String>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct ProductCompliance {
    #[serde(
        rename = "ComplianceName",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub compliance_name: Option<String>,
    #[serde(
        rename = "ComplianceValue",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub compliance_value: Option<String>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct ScheduleCartItemsRequestRoot {
    #[serde(rename = "CartKey", default, skip_serializing_if = "Option::is_none")]
    pub cart_key: Option<String>,
    #[serde(
        rename = "ScheduleCartItems",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub schedule_cart_items: Option<Vec<ScheduleReleaseRequest>>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct ScheduleRelease {
    #[serde(rename = "Errors", default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<ErrorEntity>>,
    #[serde(
        rename = "ScheduleReleaseLine",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub schedule_release_line: Option<KeyValuePairOfStringAndInt32>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct ScheduleReleaseRequest {
    #[serde(rename = "MouserPartNumber")]
    pub mouser_part_number: String,
    #[serde(
        rename = "ScheduledReleases",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub scheduled_releases: Option<Vec<KeyValuePairOfStringAndInt32>>,
}

/// Search parts by keyword and specific manufacturer. Return a maximum of 50 parts.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct SearchByKeywordMfrNameRequest {
    /// Used for filtering by a specific manufacturer. Please get the manufacturerName from the
    /// GetManufacturerList method.
    #[serde(
        rename = "manufacturerName",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub manufacturer_name: Option<String>,
    /// Used for keyword part search.
    #[serde(rename = "keyword")]
    pub keyword: String,
    /// Used to specify how many records the method should return.
    #[serde(rename = "records", default, skip_serializing_if = "Option::is_none")]
    pub records: Option<i64>,
    /// Indicates which page of the total recordset the method should return. The page number is
    /// based off of the number of records specified in the records parameter. For the pageNumber,
    /// the number of records specified will be returned up to the end of the recordset. This is
    /// useful for paging through the complete recordset of parts matching keyword.
    #[serde(
        rename = "pageNumber",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub page_number: Option<i64>,
    /// Optional. If not provided, the default is None. Refers to options supported by the search
    /// engine. Only one value at a time is supported. Available options: None | Rohs | InStock |
    /// RohsAndInStock - can use string representations or integer IDs: 1\[None\] | 2\[Rohs\] |
    /// 4\[InStock\] | 8\[RohsAndInStock\].
    #[serde(
        rename = "searchOptions",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub search_options: Option<String>,
    /// Optional. If not provided, the default is false. Used when searching for keywords in the
    /// language specified when you signed up for Search API. Can use string representation: true.
    #[serde(
        rename = "searchWithYourSignUpLanguage",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub search_with_your_sign_up_language: Option<String>,
    /// Optional. If true, prices reflect Mouser paying applicable Import Duty and Customs fees on
    /// customer's behalf. Default is false.
    #[serde(
        rename = "mouserPaysCustomsAndDuties",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub mouser_pays_customs_and_duties: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct SearchByKeywordMfrNameRequestRoot {
    #[serde(
        rename = "SearchByKeywordMfrNameRequest",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub search_by_keyword_mfr_name_request: Option<SearchByKeywordMfrNameRequest>,
}

/// Search parts by keyword and specific manufacturer. Return a maximum of 50 parts.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct SearchByKeywordMfrRequest {
    /// Used for filtering by a specific manufacturer. Please get the manufacturerId from the
    /// GetManufacturerList method.
    #[serde(
        rename = "manufacturerId",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub manufacturer_id: Option<i64>,
    /// Used for keyword part search.
    #[serde(rename = "keyword")]
    pub keyword: String,
    /// Used to specify how many records the method should return.
    #[serde(rename = "records", default, skip_serializing_if = "Option::is_none")]
    pub records: Option<i64>,
    /// Indicates where in the total recordset the return set should begin. From the startingRecord,
    /// the number of records specified will be returned up to the end of the recordset. This is
    /// useful for paging through the complete recordset of parts matching keyword.
    #[serde(
        rename = "startingRecord",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub starting_record: Option<i64>,
    /// Optional. If not provided, the default is None. Refers to options supported by the search
    /// engine. Only one value at a time is supported. Available options: None | Rohs | InStock |
    /// RohsAndInStock - can use string representations or integer IDs: 1\[None\] | 2\[Rohs\] |
    /// 4\[InStock\] | 8\[RohsAndInStock\].
    #[serde(
        rename = "searchOptions",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub search_options: Option<String>,
    /// Optional. If not provided, the default is false. Used when searching for keywords in the
    /// language specified when you signed up for Search API. Can use string representation: true.
    #[serde(
        rename = "searchWithYourSignUpLanguage",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub search_with_your_sign_up_language: Option<String>,
    /// Optional. If true, prices reflect Mouser paying applicable Import Duty and Customs fees on
    /// customer's behalf. Default is false.
    #[serde(
        rename = "mouserPaysCustomsAndDuties",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub mouser_pays_customs_and_duties: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct SearchByKeywordMfrRequestRoot {
    #[serde(
        rename = "SearchByKeywordMfrRequest",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub search_by_keyword_mfr_request: Option<SearchByKeywordMfrRequest>,
}

/// Search parts by keyword and return a maximum of 50 parts.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct SearchByKeywordRequest {
    /// Used for keyword part search.
    #[serde(rename = "keyword")]
    pub keyword: String,
    /// Used to specify how many records the method should return.
    #[serde(rename = "records", default, skip_serializing_if = "Option::is_none")]
    pub records: Option<i64>,
    /// Indicates where in the total recordset the return set should begin. From the startingRecord,
    /// the number of records specified will be returned up to the end of the recordset. This is
    /// useful for paging through the complete recordset of parts matching keyword.
    #[serde(
        rename = "startingRecord",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub starting_record: Option<i64>,
    /// Optional. If not provided, the default is None. Refers to options supported by the search
    /// engine. Only one value at a time is supported. Available options: None | Rohs | InStock |
    /// RohsAndInStock - can use string representations or integer IDs: 1\[None\] | 2\[Rohs\] |
    /// 4\[InStock\] | 8\[RohsAndInStock\].
    #[serde(
        rename = "searchOptions",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub search_options: Option<String>,
    /// Optional. If not provided, the default is false. Used when searching for keywords in the
    /// language specified when you signed up for Search API. Can use string representation: true.
    #[serde(
        rename = "searchWithYourSignUpLanguage",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub search_with_your_sign_up_language: Option<String>,
    /// Optional. If true, prices reflect Mouser paying applicable Import Duty and Customs fees on
    /// customer's behalf. Default is false.
    #[serde(
        rename = "mouserPaysCustomsAndDuties",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub mouser_pays_customs_and_duties: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct SearchByKeywordRequestRoot {
    #[serde(
        rename = "SearchByKeywordRequest",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub search_by_keyword_request: Option<SearchByKeywordRequest>,
}

/// Search parts by part number and specific manufacturer. Return a maximum of 50 parts.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct SearchByPartMfrNameRequest {
    /// Used for filtering by a specific manufacturer. Please get the manufacturerName from the
    /// GetManufacturerList method.
    #[serde(
        rename = "manufacturerName",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub manufacturer_name: Option<String>,
    /// Used to search parts by the specific Mouser part number with a maximum input of 10 part
    /// numbers, separated by a pipe symbol for the search. Each part number must be a minimum of 3
    /// characters and a maximum of 40 characters. For example:
    /// 494-JANTX2N2222A|610-2N2222-TL|637-2N2222A
    #[serde(
        rename = "mouserPartNumber",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub mouser_part_number: Option<String>,
    /// Optional. If not provided, the default is None. Refers to options supported by the search
    /// engine. Only one value at a time is supported. The following values are valid: None | Exact
    /// - can use string representations or integer IDs: 1\[None\] | 2\[Exact\]
    #[serde(
        rename = "partSearchOptions",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub part_search_options: Option<String>,
    /// Optional. If true, prices reflect Mouser paying applicable Import Duty and Customs fees on
    /// customer's behalf. Default is false.
    #[serde(
        rename = "mouserPaysCustomsAndDuties",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub mouser_pays_customs_and_duties: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct SearchByPartMfrNameRequestRoot {
    #[serde(
        rename = "SearchByPartMfrNameRequest",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub search_by_part_mfr_name_request: Option<SearchByPartMfrNameRequest>,
}

/// Search parts by part number and specific manufacturer. Return a maximum of 50 parts.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct SearchByPartMfrRequest {
    /// Used for filtering by a specific manufacturer. Please get the manufacturerId from the
    /// GetManufacturerList method.
    #[serde(
        rename = "manufacturerId",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub manufacturer_id: Option<i64>,
    /// Used to search parts by the specific Mouser part number with a maximum input of 10 part
    /// numbers, separated by a pipe symbol for the search. Each part number must be a minimum of 3
    /// characters and a maximum of 40 characters. For example:
    /// 494-JANTX2N2222A|610-2N2222-TL|637-2N2222A
    #[serde(
        rename = "mouserPartNumber",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub mouser_part_number: Option<String>,
    /// Optional. If not provided, the default is None. Refers to options supported by the search
    /// engine. Only one value at a time is supported. The following values are valid: None | Exact
    /// - can use string representations or integer IDs: 1\[None\] | 2\[Exact\]
    #[serde(
        rename = "partSearchOptions",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub part_search_options: Option<String>,
    /// Optional. If true, prices reflect Mouser paying applicable Import Duty and Customs fees on
    /// customer's behalf. Default is false.
    #[serde(
        rename = "mouserPaysCustomsAndDuties",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub mouser_pays_customs_and_duties: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct SearchByPartMfrRequestRoot {
    #[serde(
        rename = "SearchByPartMfrRequest",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub search_by_part_mfr_request: Option<SearchByPartMfrRequest>,
}

/// Search parts by part number and return a maximum of 50 parts.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct SearchByPartRequest {
    /// Used to search parts by the specific Mouser part number with a maximum input of 10 part
    /// numbers, separated by a pipe symbol for the search. Each part number must be a minimum of 3
    /// characters and a maximum of 40 characters. For example:
    /// 494-JANTX2N2222A|610-2N2222-TL|637-2N2222A
    #[serde(
        rename = "mouserPartNumber",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub mouser_part_number: Option<String>,
    /// Optional. If not provided, the default is None. Refers to options supported by the search
    /// engine. Only one value at a time is supported. The following values are valid: None | Exact
    /// - can use string representations or integer IDs: 1\[None\] | 2\[Exact\]
    #[serde(
        rename = "partSearchOptions",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub part_search_options: Option<String>,
    /// Optional. If true, prices reflect Mouser paying applicable Import Duty and Customs fees on
    /// customer's behalf. Default is false.
    #[serde(
        rename = "mouserPaysCustomsAndDuties",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub mouser_pays_customs_and_duties: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct SearchByPartRequestRoot {
    #[serde(
        rename = "SearchByPartRequest",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub search_by_part_request: Option<SearchByPartRequest>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct SearchResponse {
    #[serde(
        rename = "NumberOfResult",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub number_of_result: Option<i64>,
    #[serde(rename = "Parts", default, skip_serializing_if = "Option::is_none")]
    pub parts: Option<Vec<MouserPart>>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct SearchResponseRoot {
    #[serde(rename = "Errors", default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<ErrorEntity>>,
    #[serde(
        rename = "SearchResults",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub search_results: Option<SearchResponse>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct ShippingMethod {
    #[serde(rename = "Code", default, skip_serializing_if = "Option::is_none")]
    pub code: Option<i64>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct ShippingMethodType {
    #[serde(rename = "Method", default, skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(rename = "Rate", default, skip_serializing_if = "Option::is_none")]
    pub rate: Option<f64>,
    #[serde(rename = "Code", default, skip_serializing_if = "Option::is_none")]
    pub code: Option<i64>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct ShippingRoot {
    #[serde(rename = "Methods", default, skip_serializing_if = "Option::is_none")]
    pub methods: Option<Vec<ShippingMethodType>>,
    #[serde(
        rename = "FreightAccounts",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub freight_accounts: Option<Vec<FreightAccount>>,
    #[serde(rename = "Error", default, skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct StandardCost {
    #[serde(
        rename = "Standardcost",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub standardcost: Option<f64>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct StateResponse {
    #[serde(rename = "StateName", default, skip_serializing_if = "Option::is_none")]
    pub state_name: Option<String>,
    #[serde(rename = "StateCode", default, skip_serializing_if = "Option::is_none")]
    pub state_code: Option<String>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct SurchargeMessages {
    #[serde(rename = "code", default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(rename = "message", default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct TaxCertificates {
    #[serde(rename = "Key", default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(rename = "Name", default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct TrackingModel {
    #[serde(rename = "Number", default, skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,
    #[serde(rename = "Link", default, skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct TradeCompliance {
    #[serde(
        rename = "ComplianceName",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub compliance_name: Option<String>,
    #[serde(
        rename = "ComplianceValue",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub compliance_value: Option<String>,
    #[serde(
        rename = "ComplianceDescription",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub compliance_description: Option<String>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct UnitWeightKg {
    #[serde(
        rename = "UnitWeight",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub unit_weight: Option<f64>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct VatAccount {
    #[serde(rename = "VatNumber", default, skip_serializing_if = "Option::is_none")]
    pub vat_number: Option<String>,
    #[serde(rename = "Status", default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Fields Mouser sent that the Swagger documents do not describe.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}
